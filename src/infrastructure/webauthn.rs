use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use tokio::sync::RwLock;
use webauthn_rs::prelude::{
    DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration,
};

use crate::domain::ids::UserId;

/// Stores in-flight `WebAuthn` ceremony state between start/finish calls.
/// Entries expire after 5 minutes.
#[derive(Clone)]
pub struct ChallengeStore {
    registrations: Arc<RwLock<HashMap<String, RegistrationEntry>>>,
    authentications: Arc<RwLock<HashMap<String, AuthenticationEntry>>>,
    discoverable_authentications: Arc<RwLock<HashMap<String, DiscoverableAuthEntry>>>,
}

struct RegistrationEntry {
    pub user_id: UserId,
    pub state: PasskeyRegistration,
    pub expires_at: DateTime<Utc>,
}

struct AuthenticationEntry {
    pub state: PasskeyAuthentication,
    pub expires_at: DateTime<Utc>,
    pub cli_callback: Option<CliCallbackInfo>,
}

struct DiscoverableAuthEntry {
    pub state: DiscoverableAuthentication,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CliCallbackInfo {
    pub callback_url: String,
    pub state: String,
    pub token_name: String,
}

const CHALLENGE_TTL_MINUTES: i64 = 5;

impl Default for ChallengeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ChallengeStore {
    pub fn new() -> Self {
        Self {
            registrations: Arc::new(RwLock::new(HashMap::new())),
            authentications: Arc::new(RwLock::new(HashMap::new())),
            discoverable_authentications: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store_registration(
        &self,
        challenge_id: String,
        user_id: UserId,
        state: PasskeyRegistration,
    ) {
        let entry = RegistrationEntry {
            user_id,
            state,
            expires_at: Utc::now() + Duration::minutes(CHALLENGE_TTL_MINUTES),
        };
        let mut map = self.registrations.write().await;
        Self::cleanup_expired_registrations(&mut map);
        map.insert(challenge_id, entry);
    }

    pub async fn take_registration(
        &self,
        challenge_id: &str,
    ) -> Option<(UserId, PasskeyRegistration)> {
        let mut map = self.registrations.write().await;
        let entry = map.remove(challenge_id)?;
        if Utc::now() > entry.expires_at {
            return None;
        }
        Some((entry.user_id, entry.state))
    }

    pub async fn store_authentication(
        &self,
        challenge_id: String,
        state: PasskeyAuthentication,
        cli_callback: Option<CliCallbackInfo>,
    ) {
        let entry = AuthenticationEntry {
            state,
            expires_at: Utc::now() + Duration::minutes(CHALLENGE_TTL_MINUTES),
            cli_callback,
        };
        let mut map = self.authentications.write().await;
        Self::cleanup_expired_authentications(&mut map);
        map.insert(challenge_id, entry);
    }

    pub async fn take_authentication(
        &self,
        challenge_id: &str,
    ) -> Option<(PasskeyAuthentication, Option<CliCallbackInfo>)> {
        let mut map = self.authentications.write().await;
        let entry = map.remove(challenge_id)?;
        if Utc::now() > entry.expires_at {
            return None;
        }
        Some((entry.state, entry.cli_callback))
    }

    pub async fn store_discoverable_authentication(
        &self,
        challenge_id: String,
        state: DiscoverableAuthentication,
    ) {
        let entry = DiscoverableAuthEntry {
            state,
            expires_at: Utc::now() + Duration::minutes(CHALLENGE_TTL_MINUTES),
        };
        let mut map = self.discoverable_authentications.write().await;
        Self::cleanup_expired_discoverable(&mut map);
        map.insert(challenge_id, entry);
    }

    pub async fn take_discoverable_authentication(
        &self,
        challenge_id: &str,
    ) -> Option<DiscoverableAuthentication> {
        let mut map = self.discoverable_authentications.write().await;
        let entry = map.remove(challenge_id)?;
        if Utc::now() > entry.expires_at {
            return None;
        }
        Some(entry.state)
    }

    fn cleanup_expired_registrations(map: &mut HashMap<String, RegistrationEntry>) {
        let now = Utc::now();
        map.retain(|_, entry| entry.expires_at > now);
    }

    fn cleanup_expired_authentications(map: &mut HashMap<String, AuthenticationEntry>) {
        let now = Utc::now();
        map.retain(|_, entry| entry.expires_at > now);
    }

    fn cleanup_expired_discoverable(map: &mut HashMap<String, DiscoverableAuthEntry>) {
        let now = Utc::now();
        map.retain(|_, entry| entry.expires_at > now);
    }
}
