use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info, warn};
use uuid::Uuid;
use webauthn_rs::prelude::*;
use webauthn_rs_proto::ResidentKeyRequirement;

use crate::application::auth::{AuthenticatedUser, SESSION_COOKIE_NAME};
use crate::application::state::AppState;
use crate::domain::passkey_credentials::NewPasskeyCredential;
use crate::domain::sessions::NewSession;
use crate::domain::tokens::NewToken;
use crate::domain::users::NewUser;
use crate::infrastructure::auth::{generate_session_token, generate_token, hash_token};
use crate::infrastructure::webauthn::CliCallbackInfo;

// --- Request/Response types ---

#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub token: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse<T: Serialize> {
    pub challenge_id: String,
    pub options: T,
}

#[derive(Deserialize)]
pub struct RegisterFinishRequest {
    pub challenge_id: String,
    pub passkey_name: String,
    pub credential: RegisterPublicKeyCredential,
}

#[derive(Debug, Deserialize)]
pub struct AuthStartQuery {
    pub cli_callback: Option<String>,
    pub state: Option<String>,
    pub token_name: Option<String>,
}

#[derive(Serialize)]
pub struct AuthStartResponse {
    pub challenge_id: String,
    pub options: RequestChallengeResponse,
}

#[derive(Deserialize)]
pub struct AuthFinishRequest {
    pub challenge_id: String,
    pub credential: PublicKeyCredential,
}

#[derive(Serialize)]
pub struct AuthFinishResponse {
    pub redirect: Option<String>,
}

// --- Registration start (creates user + begins ceremony) ---

#[tracing::instrument(skip(state, payload), fields(display_name = %payload.display_name))]
pub(crate) async fn register_start(
    State(state): State<AppState>,
    Json(payload): Json<RegisterStartRequest>,
) -> Result<Json<ChallengeResponse<CreationChallengeResponse>>, StatusCode> {
    // Validate registration token
    let token_hash = hash_token(&payload.token);
    let reg_token = state
        .registration_token_repo
        .get_by_token_hash(&token_hash)
        .await
        .map_err(|err| {
            warn!(error = %err, "registration token lookup failed");
            StatusCode::UNAUTHORIZED
        })?;

    if !reg_token.is_valid() {
        return Err(StatusCode::GONE);
    }

    // Create the user
    let user_uuid = Uuid::new_v4().to_string();
    let new_user = NewUser::new(payload.display_name, user_uuid.clone());
    if let Err(msg) = new_user.validate() {
        warn!(error = msg, "invalid username during registration");
        return Err(StatusCode::BAD_REQUEST);
    }
    let user = state.user_repo.insert(new_user).await.map_err(|err| {
        error!(error = %err, "failed to create user during registration");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Mark registration token as used
    if let Err(err) = state
        .registration_token_repo
        .mark_used(reg_token.id, user.id)
        .await
    {
        warn!(error = %err, token_id = %reg_token.id, "failed to mark registration token as used");
    }

    // Start passkey registration ceremony
    let webauthn_uuid = Uuid::parse_str(&user_uuid).map_err(|err| {
        error!(error = %err, "failed to parse user UUID");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let exclude_credentials = Vec::new();

    let (mut ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(
            webauthn_uuid,
            &user.username,
            &user.username,
            Some(exclude_credentials),
        )
        .map_err(|err| {
            error!(error = %err, "failed to start passkey registration");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    require_discoverable_credential(&mut ccr);

    // Store ceremony state
    let challenge_id = generate_session_token();
    state
        .challenge_store
        .store_registration(challenge_id.clone(), user.id, reg_state)
        .await;

    Ok(Json(ChallengeResponse {
        challenge_id,
        options: ccr,
    }))
}

// --- Registration finish ---

#[tracing::instrument(skip(state, cookies, payload))]
pub(crate) async fn register_finish(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<RegisterFinishRequest>,
) -> Result<Json<AuthFinishResponse>, StatusCode> {
    // Retrieve ceremony state
    let (user_id, reg_state) = state
        .challenge_store
        .take_registration(&payload.challenge_id)
        .await
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Complete registration
    let passkey = state
        .webauthn
        .finish_passkey_registration(&payload.credential, &reg_state)
        .map_err(|err| {
            warn!(error = %err, "passkey registration failed");
            StatusCode::BAD_REQUEST
        })?;

    // Store the credential
    let credential_json = serde_json::to_string(&passkey).map_err(|err| {
        error!(error = %err, "failed to serialize passkey credential");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let new_credential = NewPasskeyCredential::new(user_id, credential_json, payload.passkey_name);
    state
        .passkey_repo
        .insert(new_credential)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to store passkey credential");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(user_id = %user_id, "passkey registered successfully");

    // Create session for the new user
    create_session(&state, &cookies, user_id).await;

    Ok(Json(AuthFinishResponse { redirect: None }))
}

// --- Authentication start ---

#[tracing::instrument(skip(state))]
pub(crate) async fn auth_start(
    State(state): State<AppState>,
    Query(query): Query<AuthStartQuery>,
) -> Result<Json<AuthStartResponse>, StatusCode> {
    // Load all passkey credentials in a single query
    let credentials = state.passkey_repo.list_all().await.map_err(|err| {
        error!(error = %err, "failed to list all passkey credentials");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut all_passkeys: Vec<Passkey> = Vec::new();
    for cred in credentials {
        let passkey: Passkey = serde_json::from_str(&cred.credential_json).map_err(|err| {
            error!(error = %err, credential_id = %cred.id, "failed to deserialize passkey credential");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        all_passkeys.push(passkey);
    }

    if all_passkeys.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let (rcr, auth_state) = state
        .webauthn
        .start_passkey_authentication(&all_passkeys)
        .map_err(|err| {
            error!(error = %err, "failed to start passkey authentication");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let cli_callback = match (query.cli_callback, query.state, query.token_name) {
        (Some(callback_url), Some(cli_state), Some(token_name)) => {
            validate_cli_callback_url(&callback_url).map_err(|()| {
                warn!(url = %callback_url, "rejected non-localhost CLI callback URL");
                StatusCode::BAD_REQUEST
            })?;
            Some(CliCallbackInfo {
                callback_url,
                state: cli_state,
                token_name,
            })
        }
        _ => None,
    };

    let challenge_id = generate_session_token();
    state
        .challenge_store
        .store_authentication(challenge_id.clone(), auth_state, cli_callback)
        .await;

    Ok(Json(AuthStartResponse {
        challenge_id,
        options: rcr,
    }))
}

// --- Authentication finish ---

#[allow(clippy::too_many_lines)]
#[tracing::instrument(skip(state, cookies, payload))]
pub(crate) async fn auth_finish(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<AuthFinishRequest>,
) -> Result<Json<AuthFinishResponse>, StatusCode> {
    // Retrieve ceremony state
    let (auth_state, cli_callback) = state
        .challenge_store
        .take_authentication(&payload.challenge_id)
        .await
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Complete authentication
    let auth_result = state
        .webauthn
        .finish_passkey_authentication(&payload.credential, &auth_state)
        .map_err(|err| {
            warn!(error = %err, "passkey authentication failed");
            StatusCode::UNAUTHORIZED
        })?;

    // Find the user who owns this credential
    let credential_id = auth_result.cred_id();
    let credentials = state.passkey_repo.list_all().await.map_err(|err| {
        error!(error = %err, "failed to list passkey credentials for credential lookup");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut found_user_id = None;
    let mut found_cred_id = None;
    let mut found_passkey: Option<Passkey> = None;

    for cred in &credentials {
        let passkey: Passkey = serde_json::from_str(&cred.credential_json)
            .map_err(|err| {
                error!(error = %err, credential_id = %cred.id, "failed to deserialize passkey credential");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        if passkey.cred_id() == credential_id {
            found_user_id = Some(cred.user_id);
            found_cred_id = Some(cred.id);
            found_passkey = Some(passkey);
            break;
        }
    }

    let user_id = found_user_id.ok_or(StatusCode::UNAUTHORIZED)?;
    let cred_db_id = found_cred_id.ok_or(StatusCode::UNAUTHORIZED)?;

    // Update credential counter if needed
    if auth_result.needs_update()
        && let Some(mut passkey) = found_passkey
    {
        passkey.update_credential(&auth_result);
        match serde_json::to_string(&passkey) {
            Ok(updated_json) => {
                if let Err(err) = state
                    .passkey_repo
                    .update_credential_json(cred_db_id, &updated_json)
                    .await
                {
                    warn!(error = %err, credential_id = %cred_db_id, "failed to update passkey credential counter");
                }
            }
            Err(err) => {
                warn!(error = %err, "failed to serialize updated passkey credential");
            }
        }
    }

    // Update last used timestamp
    let passkey_repo = state.passkey_repo.clone();
    tokio::spawn(async move {
        if let Err(err) = passkey_repo.update_last_used(cred_db_id).await {
            warn!(error = %err, credential_id = %cred_db_id, "failed to update passkey last_used");
        }
    });

    // Handle CLI callback flow
    if let Some(cli_info) = cli_callback {
        // Generate a bearer token for the CLI
        let token_value = generate_token().map_err(|err| {
            error!(error = %err, "failed to generate CLI bearer token");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let token_hash_value = hash_token(&token_value);
        let new_token = NewToken::new(user_id, token_hash_value, cli_info.token_name);
        state.token_repo.insert(new_token).await.map_err(|err| {
            error!(error = %err, "failed to store CLI bearer token");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        info!(user_id = %user_id, "CLI token created via passkey auth");

        let redirect_url = format!(
            "{}?token={}&state={}",
            cli_info.callback_url, token_value, cli_info.state
        );
        return Ok(Json(AuthFinishResponse {
            redirect: Some(redirect_url),
        }));
    }

    // Normal web login: create session
    create_session(&state, &cookies, user_id).await;

    info!(user_id = %user_id, "user authenticated via passkey");

    Ok(Json(AuthFinishResponse { redirect: None }))
}

// --- Add passkey to existing account ---

#[derive(Deserialize)]
pub struct PasskeyAddStartRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct PasskeyAddFinishRequest {
    pub challenge_id: String,
    pub name: String,
    pub credential: RegisterPublicKeyCredential,
}

#[tracing::instrument(skip(state, auth_user, payload), fields(passkey_name = %payload.name))]
pub(crate) async fn passkey_add_start(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<PasskeyAddStartRequest>,
) -> Result<Json<ChallengeResponse<CreationChallengeResponse>>, StatusCode> {
    let user = auth_user.0;
    let webauthn_uuid = Uuid::parse_str(&user.uuid).map_err(|err| {
        error!(error = %err, "failed to parse user UUID for passkey add");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Load existing credentials to exclude (prevents re-registering same authenticator)
    let existing = state
        .passkey_repo
        .list_by_user(user.id)
        .await
        .map_err(|err| {
            error!(error = %err, user_id = %user.id, "failed to list existing passkeys");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let exclude_credentials = existing
        .iter()
        .filter_map(|c| {
            serde_json::from_str::<Passkey>(&c.credential_json)
                .map_err(|err| {
                    warn!(error = %err, credential_id = %c.id, "failed to deserialize passkey credential for exclude list");
                })
                .ok()
        })
        .map(|p| p.cred_id().clone())
        .collect::<Vec<_>>();

    let (mut ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(
            webauthn_uuid,
            &user.username,
            &user.username,
            Some(exclude_credentials),
        )
        .map_err(|err| {
            error!(error = %err, "failed to start passkey registration for existing user");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    require_discoverable_credential(&mut ccr);

    let challenge_id = generate_session_token();
    state
        .challenge_store
        .store_registration(challenge_id.clone(), user.id, reg_state)
        .await;

    Ok(Json(ChallengeResponse {
        challenge_id,
        options: ccr,
    }))
}

#[tracing::instrument(skip(state, auth_user, payload))]
pub(crate) async fn passkey_add_finish(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(payload): Json<PasskeyAddFinishRequest>,
) -> Result<StatusCode, StatusCode> {
    let (user_id, reg_state) = state
        .challenge_store
        .take_registration(&payload.challenge_id)
        .await
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Verify the authenticated user matches the challenge owner
    if auth_user.0.id != user_id {
        warn!(
            auth_user_id = %auth_user.0.id,
            challenge_user_id = %user_id,
            "passkey add finish: user mismatch"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    let passkey = state
        .webauthn
        .finish_passkey_registration(&payload.credential, &reg_state)
        .map_err(|err| {
            warn!(error = %err, "passkey add registration failed");
            StatusCode::BAD_REQUEST
        })?;

    let credential_json = serde_json::to_string(&passkey).map_err(|err| {
        error!(error = %err, "failed to serialize new passkey credential");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let new_credential = NewPasskeyCredential::new(user_id, credential_json, payload.name);
    state
        .passkey_repo
        .insert(new_credential)
        .await
        .map_err(|err| {
            error!(error = %err, "failed to store new passkey credential");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(user_id = %user_id, "additional passkey registered successfully");

    Ok(StatusCode::OK)
}

// --- Discoverable authentication (Conditional UI) ---

#[tracing::instrument(skip(state))]
pub(crate) async fn discoverable_auth_start(
    State(state): State<AppState>,
) -> Result<Json<AuthStartResponse>, StatusCode> {
    let (rcr, auth_state) = state
        .webauthn
        .start_discoverable_authentication()
        .map_err(|err| {
            error!(error = %err, "failed to start discoverable authentication");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let challenge_id = generate_session_token();
    state
        .challenge_store
        .store_discoverable_authentication(challenge_id.clone(), auth_state)
        .await;

    Ok(Json(AuthStartResponse {
        challenge_id,
        options: rcr,
    }))
}

#[tracing::instrument(skip(state, cookies, payload))]
pub(crate) async fn discoverable_auth_finish(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(payload): Json<AuthFinishRequest>,
) -> Result<Json<AuthFinishResponse>, StatusCode> {
    let auth_state = state
        .challenge_store
        .take_discoverable_authentication(&payload.challenge_id)
        .await
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Extract user UUID and credential ID from the credential
    let (user_uuid, _credential_id) = state
        .webauthn
        .identify_discoverable_authentication(&payload.credential)
        .map_err(|err| {
            warn!(error = %err, "failed to identify discoverable credential");
            StatusCode::UNAUTHORIZED
        })?;

    // Look up user by UUID
    let user = state
        .user_repo
        .get_by_uuid(&user_uuid.to_string())
        .await
        .map_err(|err| {
            warn!(error = %err, uuid = %user_uuid, "user not found for discoverable auth");
            StatusCode::UNAUTHORIZED
        })?;

    // Load user's passkeys and convert to DiscoverableKey
    let credentials = state
        .passkey_repo
        .list_by_user(user.id)
        .await
        .map_err(|err| {
            error!(error = %err, user_id = %user.id, "failed to list passkeys for discoverable auth");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut discoverable_keys: Vec<DiscoverableKey> = Vec::new();
    for cred in &credentials {
        let passkey: Passkey = serde_json::from_str(&cred.credential_json).map_err(|err| {
            error!(error = %err, credential_id = %cred.id, "failed to deserialize passkey for discoverable auth");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        discoverable_keys.push(DiscoverableKey::from(&passkey));
    }

    // Complete discoverable authentication
    let auth_result = state
        .webauthn
        .finish_discoverable_authentication(&payload.credential, auth_state, &discoverable_keys)
        .map_err(|err| {
            warn!(error = %err, "discoverable authentication failed");
            StatusCode::UNAUTHORIZED
        })?;

    // Update credential counter if needed
    let credential_id = auth_result.cred_id();
    for cred in &credentials {
        let passkey: Passkey = match serde_json::from_str(&cred.credential_json) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if passkey.cred_id() == credential_id && auth_result.needs_update() {
            let mut updated_passkey = passkey;
            updated_passkey.update_credential(&auth_result);
            match serde_json::to_string(&updated_passkey) {
                Ok(updated_json) => {
                    if let Err(err) = state
                        .passkey_repo
                        .update_credential_json(cred.id, &updated_json)
                        .await
                    {
                        warn!(error = %err, credential_id = %cred.id, "failed to update passkey credential counter");
                    }
                }
                Err(err) => {
                    warn!(error = %err, "failed to serialize updated passkey credential");
                }
            }

            // Update last used timestamp
            let passkey_repo = state.passkey_repo.clone();
            let cred_id = cred.id;
            tokio::spawn(async move {
                if let Err(err) = passkey_repo.update_last_used(cred_id).await {
                    warn!(error = %err, credential_id = %cred_id, "failed to update passkey last_used");
                }
            });
            break;
        }
    }

    create_session(&state, &cookies, user.id).await;

    info!(user_id = %user.id, "user authenticated via discoverable passkey");

    Ok(Json(AuthFinishResponse { redirect: None }))
}

// --- Helpers ---

/// Patch the creation challenge to require a discoverable (resident) credential.
///
/// The webauthn-rs `start_passkey_registration` sets `residentKey: "discouraged"`,
/// which prevents iOS Safari from creating discoverable passkeys. Desktop password
/// managers ignore this and create discoverable credentials anyway, but iOS respects
/// it strictly. The server-side `RegistrationState` discards `require_resident_key`
/// during `finish_passkey_registration`, so this client-only patch is safe.
fn require_discoverable_credential(ccr: &mut CreationChallengeResponse) {
    if let Some(ref mut auth_sel) = ccr.public_key.authenticator_selection {
        auth_sel.resident_key = Some(ResidentKeyRequirement::Required);
        auth_sel.require_resident_key = true;
    }
}

/// Reject CLI callback URLs that don't point to localhost.
/// This prevents an attacker from redirecting the bearer token to an external server.
fn validate_cli_callback_url(url_str: &str) -> Result<(), ()> {
    let parsed = url::Url::parse(url_str).map_err(|_| ())?;
    if parsed.scheme() != "http" {
        return Err(());
    }
    match parsed.host_str() {
        Some("127.0.0.1" | "localhost" | "::1" | "[::1]") => Ok(()),
        _ => Err(()),
    }
}

async fn create_session(state: &AppState, cookies: &Cookies, user_id: crate::domain::ids::UserId) {
    let session_token = generate_session_token();
    let session_token_hash = hash_token(&session_token);

    let new_session = NewSession::new(
        user_id,
        session_token_hash,
        Utc::now(),
        Utc::now() + Duration::days(30),
    );

    if let Err(err) = state.session_repo.insert(new_session).await {
        error!(error = %err, "failed to create session");
        return;
    }

    let mut cookie = Cookie::new(SESSION_COOKIE_NAME, session_token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_max_age(tower_cookies::cookie::time::Duration::days(30));

    if !state.insecure_cookies {
        cookie.set_secure(true);
    }

    cookies.add(cookie);
}
