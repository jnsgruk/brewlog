use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use chrono::{Duration, Utc};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;
use webauthn_rs::prelude::*;

use crate::application::routes::app_router;
use crate::domain::registration_tokens::NewRegistrationToken;
use crate::domain::repositories::{
    BagRepository, BrewRepository, CafeRepository, CupRepository, GearRepository,
    PasskeyCredentialRepository, RegistrationTokenRepository, RoastRepository, RoasterRepository,
    SessionRepository, TimelineEventRepository, TokenRepository, UserRepository,
};
use crate::infrastructure::auth::{generate_session_token, hash_token};
use crate::infrastructure::backup::BackupService;
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::bags::SqlBagRepository;
use crate::infrastructure::repositories::brews::SqlBrewRepository;
use crate::infrastructure::repositories::cafes::SqlCafeRepository;
use crate::infrastructure::repositories::cups::SqlCupRepository;
use crate::infrastructure::repositories::gear::SqlGearRepository;
use crate::infrastructure::repositories::passkey_credentials::SqlPasskeyCredentialRepository;
use crate::infrastructure::repositories::registration_tokens::SqlRegistrationTokenRepository;
use crate::infrastructure::repositories::roasters::SqlRoasterRepository;
use crate::infrastructure::repositories::roasts::SqlRoastRepository;
use crate::infrastructure::repositories::sessions::SqlSessionRepository;
use crate::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;
use crate::infrastructure::repositories::tokens::SqlTokenRepository;
use crate::infrastructure::repositories::users::SqlUserRepository;
use crate::infrastructure::webauthn::ChallengeStore;

pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub database_url: String,
    pub rp_id: String,
    pub rp_origin: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub foursquare_api_key: String,
}

#[derive(Clone)]
pub struct AppState {
    pub roaster_repo: Arc<dyn RoasterRepository>,
    pub roast_repo: Arc<dyn RoastRepository>,
    pub bag_repo: Arc<dyn BagRepository>,
    pub gear_repo: Arc<dyn GearRepository>,
    pub brew_repo: Arc<dyn BrewRepository>,
    pub cafe_repo: Arc<dyn CafeRepository>,
    pub cup_repo: Arc<dyn CupRepository>,
    pub timeline_repo: Arc<dyn TimelineEventRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub token_repo: Arc<dyn TokenRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub passkey_repo: Arc<dyn PasskeyCredentialRepository>,
    pub registration_token_repo: Arc<dyn RegistrationTokenRepository>,
    pub webauthn: Arc<Webauthn>,
    pub challenge_store: Arc<ChallengeStore>,
    pub http_client: reqwest::Client,
    pub foursquare_url: String,
    pub foursquare_api_key: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub backup_service: Arc<BackupService>,
}

pub async fn serve(config: ServerConfig) -> anyhow::Result<()> {
    let database = Database::connect(&config.database_url)
        .await
        .context("failed to connect to database")?;

    let rp_origin = url::Url::parse(&config.rp_origin).context("invalid BREWLOG_RP_ORIGIN URL")?;
    let webauthn = Arc::new(
        WebauthnBuilder::new(&config.rp_id, &rp_origin)
            .context("failed to build WebAuthn instance")?
            .rp_name("Brewlog")
            .build()
            .context("failed to build WebAuthn instance")?,
    );

    let roaster_repo = Arc::new(SqlRoasterRepository::new(database.clone_pool()));
    let roast_repo = Arc::new(SqlRoastRepository::new(database.clone_pool()));
    let bag_repo = Arc::new(SqlBagRepository::new(database.clone_pool()));
    let gear_repo = Arc::new(SqlGearRepository::new(database.clone_pool()));
    let brew_repo = Arc::new(SqlBrewRepository::new(database.clone_pool()));
    let cafe_repo = Arc::new(SqlCafeRepository::new(database.clone_pool()));
    let cup_repo = Arc::new(SqlCupRepository::new(database.clone_pool()));
    let timeline_repo = Arc::new(SqlTimelineEventRepository::new(database.clone_pool()));
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(SqlUserRepository::new(database.clone_pool()));
    let token_repo: Arc<dyn TokenRepository> =
        Arc::new(SqlTokenRepository::new(database.clone_pool()));
    let session_repo: Arc<dyn SessionRepository> =
        Arc::new(SqlSessionRepository::new(database.clone_pool()));
    let passkey_repo: Arc<dyn PasskeyCredentialRepository> =
        Arc::new(SqlPasskeyCredentialRepository::new(database.clone_pool()));
    let registration_token_repo: Arc<dyn RegistrationTokenRepository> =
        Arc::new(SqlRegistrationTokenRepository::new(database.clone_pool()));

    let backup_service = Arc::new(BackupService::new(database.clone_pool()));
    let challenge_store = Arc::new(ChallengeStore::new());

    // Bootstrap: if no users exist, generate a one-time registration token
    bootstrap_registration(&registration_token_repo, &user_repo, &config.rp_origin).await?;

    let state = AppState {
        roaster_repo,
        roast_repo,
        bag_repo,
        gear_repo,
        brew_repo,
        cafe_repo,
        cup_repo,
        timeline_repo,
        user_repo,
        token_repo,
        session_repo,
        passkey_repo,
        registration_token_repo,
        webauthn,
        challenge_store,
        http_client: reqwest::Client::new(),
        foursquare_url: crate::infrastructure::foursquare::FOURSQUARE_SEARCH_URL.to_string(),
        foursquare_api_key: config.foursquare_api_key,
        openrouter_api_key: config.openrouter_api_key,
        openrouter_model: config.openrouter_model,
        backup_service,
    };

    let listener = TcpListener::bind(config.bind_address)
        .await
        .with_context(|| format!("failed to bind to {}", config.bind_address))?;

    let app: Router = app_router(state);

    info!(
        address = %config.bind_address,
        database = %config.database_url,
        "starting HTTP server"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server terminated unexpectedly")?;

    info!("server shutdown complete");

    Ok(())
}

async fn bootstrap_registration(
    registration_token_repo: &Arc<dyn RegistrationTokenRepository>,
    user_repo: &Arc<dyn UserRepository>,
    rp_origin: &str,
) -> anyhow::Result<()> {
    let users_exist = user_repo
        .exists()
        .await
        .context("failed to check if users exist")?;

    if users_exist {
        return Ok(());
    }

    // Generate one-time registration token
    let token = generate_session_token();
    let token_hash = hash_token(&token);
    let now = Utc::now();
    #[allow(clippy::expect_used)]
    let expires_at = now
        .checked_add_signed(Duration::hours(1))
        .expect("timestamp overflow adding 1 hour");

    let new_token = NewRegistrationToken::new(token_hash, now, expires_at);

    registration_token_repo
        .insert(new_token)
        .await
        .context("failed to create registration token")?;

    info!("No users found. Register the first user at:");
    info!("  {}/register/{}", rp_origin, token);
    info!("This link expires in 1 hour.");

    Ok(())
}

#[allow(clippy::expect_used)] // Startup: panicking is appropriate if signal handlers fail
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
