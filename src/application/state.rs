use std::sync::Arc;

use webauthn_rs::prelude::*;

use crate::application::services::{
    BagService, BrewService, CafeService, CupService, GearService, RoastService, RoasterService,
    StatsInvalidator, TimelineInvalidator,
};
use crate::domain::repositories::{
    AiUsageRepository, BagRepository, BrewRepository, CafeRepository, CupRepository,
    GearRepository, ImageRepository, PasskeyCredentialRepository, RegistrationTokenRepository,
    RoastRepository, RoasterRepository, SessionRepository, StatsRepository,
    TimelineEventRepository, TokenRepository, UserRepository,
};
use crate::infrastructure::backup::BackupService;
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::ai_usage::SqlAiUsageRepository;
use crate::infrastructure::repositories::bags::SqlBagRepository;
use crate::infrastructure::repositories::brews::SqlBrewRepository;
use crate::infrastructure::repositories::cafes::SqlCafeRepository;
use crate::infrastructure::repositories::cups::SqlCupRepository;
use crate::infrastructure::repositories::gear::SqlGearRepository;
use crate::infrastructure::repositories::images::SqlImageRepository;
use crate::infrastructure::repositories::passkey_credentials::SqlPasskeyCredentialRepository;
use crate::infrastructure::repositories::registration_tokens::SqlRegistrationTokenRepository;
use crate::infrastructure::repositories::roasters::SqlRoasterRepository;
use crate::infrastructure::repositories::roasts::SqlRoastRepository;
use crate::infrastructure::repositories::sessions::SqlSessionRepository;
use crate::infrastructure::repositories::stats::SqlStatsRepository;
use crate::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;
use crate::infrastructure::repositories::tokens::SqlTokenRepository;
use crate::infrastructure::repositories::users::SqlUserRepository;
use crate::infrastructure::webauthn::ChallengeStore;

/// Configuration for external services and auth — everything that varies
/// between production and test environments. Repos and services are created
/// automatically from the database pool.
pub struct AppStateConfig {
    pub webauthn: Arc<Webauthn>,
    pub insecure_cookies: bool,
    pub foursquare_url: String,
    pub foursquare_api_key: String,
    pub openrouter_url: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub stats_invalidator: StatsInvalidator,
    pub timeline_invalidator: TimelineInvalidator,
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
    pub ai_usage_repo: Arc<dyn AiUsageRepository>,
    pub image_repo: Arc<dyn ImageRepository>,
    pub stats_repo: Arc<dyn StatsRepository>,
    pub webauthn: Arc<Webauthn>,
    pub challenge_store: Arc<ChallengeStore>,
    pub http_client: reqwest::Client,
    pub foursquare_url: String,
    pub foursquare_api_key: String,
    pub openrouter_url: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub backup_service: Arc<BackupService>,
    pub roaster_service: RoasterService,
    pub roast_service: RoastService,
    pub bag_service: BagService,
    pub brew_service: BrewService,
    pub gear_service: GearService,
    pub cafe_service: CafeService,
    pub cup_service: CupService,
    pub insecure_cookies: bool,
    pub stats_invalidator: StatsInvalidator,
    pub timeline_invalidator: TimelineInvalidator,
    pub image_semaphore: Arc<tokio::sync::Semaphore>,
}

impl AppState {
    /// Build the full application state from a database connection and config.
    /// Creates all repositories and services internally.
    pub fn from_database(database: &Database, config: AppStateConfig) -> Self {
        let pool = database.clone_pool();

        let roaster_repo: Arc<dyn RoasterRepository> =
            Arc::new(SqlRoasterRepository::new(pool.clone()));
        let roast_repo: Arc<dyn RoastRepository> = Arc::new(SqlRoastRepository::new(pool.clone()));
        let bag_repo: Arc<dyn BagRepository> = Arc::new(SqlBagRepository::new(pool.clone()));
        let gear_repo: Arc<dyn GearRepository> = Arc::new(SqlGearRepository::new(pool.clone()));
        let brew_repo: Arc<dyn BrewRepository> = Arc::new(SqlBrewRepository::new(pool.clone()));
        let cafe_repo: Arc<dyn CafeRepository> = Arc::new(SqlCafeRepository::new(pool.clone()));
        let cup_repo: Arc<dyn CupRepository> = Arc::new(SqlCupRepository::new(pool.clone()));
        let timeline_repo: Arc<dyn TimelineEventRepository> =
            Arc::new(SqlTimelineEventRepository::new(pool.clone()));
        let user_repo: Arc<dyn UserRepository> = Arc::new(SqlUserRepository::new(pool.clone()));
        let token_repo: Arc<dyn TokenRepository> = Arc::new(SqlTokenRepository::new(pool.clone()));
        let session_repo: Arc<dyn SessionRepository> =
            Arc::new(SqlSessionRepository::new(pool.clone()));
        let passkey_repo: Arc<dyn PasskeyCredentialRepository> =
            Arc::new(SqlPasskeyCredentialRepository::new(pool.clone()));
        let registration_token_repo: Arc<dyn RegistrationTokenRepository> =
            Arc::new(SqlRegistrationTokenRepository::new(pool.clone()));
        let ai_usage_repo: Arc<dyn AiUsageRepository> =
            Arc::new(SqlAiUsageRepository::new(pool.clone()));
        let image_repo: Arc<dyn ImageRepository> = Arc::new(SqlImageRepository::new(pool.clone()));
        let stats_repo: Arc<dyn StatsRepository> = Arc::new(SqlStatsRepository::new(pool.clone()));

        let backup_service = Arc::new(BackupService::new(pool));

        let roaster_service =
            RoasterService::new(Arc::clone(&roaster_repo), Arc::clone(&timeline_repo));
        let roast_service = RoastService::new(
            Arc::clone(&roast_repo),
            Arc::clone(&roaster_repo),
            Arc::clone(&timeline_repo),
        );
        let bag_service = BagService::new(
            Arc::clone(&bag_repo),
            Arc::clone(&roast_repo),
            Arc::clone(&roaster_repo),
            Arc::clone(&timeline_repo),
        );
        let brew_service = BrewService::new(Arc::clone(&brew_repo), Arc::clone(&timeline_repo));
        let gear_service = GearService::new(Arc::clone(&gear_repo), Arc::clone(&timeline_repo));
        let cafe_service = CafeService::new(Arc::clone(&cafe_repo), Arc::clone(&timeline_repo));
        let cup_service = CupService::new(Arc::clone(&cup_repo), Arc::clone(&timeline_repo));

        Self {
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
            ai_usage_repo,
            image_repo,
            stats_repo,
            webauthn: config.webauthn,
            challenge_store: Arc::new(ChallengeStore::new()),
            #[allow(clippy::expect_used)]
            http_client: reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to build HTTP client"),
            foursquare_url: config.foursquare_url,
            foursquare_api_key: config.foursquare_api_key,
            openrouter_url: config.openrouter_url,
            openrouter_api_key: config.openrouter_api_key,
            openrouter_model: config.openrouter_model,
            backup_service,
            roaster_service,
            roast_service,
            bag_service,
            brew_service,
            gear_service,
            cafe_service,
            cup_service,
            insecure_cookies: config.insecure_cookies,
            stats_invalidator: config.stats_invalidator,
            timeline_invalidator: config.timeline_invalidator,
            image_semaphore: Arc::new(tokio::sync::Semaphore::new(4)),
        }
    }
}
