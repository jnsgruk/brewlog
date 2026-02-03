use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::application::routes::app_router;
use crate::domain::repositories::{
    BagRepository, BrewRepository, CafeRepository, CupRepository, GearRepository, RoastRepository,
    RoasterRepository, SessionRepository, TimelineEventRepository, TokenRepository, UserRepository,
};
use crate::domain::users::NewUser;
use crate::infrastructure::auth::hash_password;
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::bags::SqlBagRepository;
use crate::infrastructure::repositories::brews::SqlBrewRepository;
use crate::infrastructure::repositories::cafes::SqlCafeRepository;
use crate::infrastructure::repositories::cups::SqlCupRepository;
use crate::infrastructure::repositories::gear::SqlGearRepository;
use crate::infrastructure::repositories::roasters::SqlRoasterRepository;
use crate::infrastructure::repositories::roasts::SqlRoastRepository;
use crate::infrastructure::repositories::sessions::SqlSessionRepository;
use crate::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;
use crate::infrastructure::repositories::tokens::SqlTokenRepository;
use crate::infrastructure::repositories::users::SqlUserRepository;

pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub database_url: String,
    pub admin_password: Option<String>,
    pub admin_username: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: String,
    pub foursquare_api_key: Option<String>,
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
    pub http_client: reqwest::Client,
    pub foursquare_url: String,
    pub foursquare_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: String,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        roaster_repo: Arc<dyn RoasterRepository>,
        roast_repo: Arc<dyn RoastRepository>,
        bag_repo: Arc<dyn BagRepository>,
        gear_repo: Arc<dyn GearRepository>,
        brew_repo: Arc<dyn BrewRepository>,
        cafe_repo: Arc<dyn CafeRepository>,
        cup_repo: Arc<dyn CupRepository>,
        timeline_repo: Arc<dyn TimelineEventRepository>,
        user_repo: Arc<dyn UserRepository>,
        token_repo: Arc<dyn TokenRepository>,
        session_repo: Arc<dyn SessionRepository>,
        http_client: reqwest::Client,
        foursquare_url: String,
        foursquare_api_key: Option<String>,
        openrouter_api_key: Option<String>,
        openrouter_model: String,
    ) -> Self {
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
            http_client,
            foursquare_url,
            foursquare_api_key,
            openrouter_api_key,
            openrouter_model,
        }
    }

    pub fn has_ai_extract(&self) -> bool {
        self.openrouter_api_key.is_some()
    }
}

pub async fn serve(config: ServerConfig) -> anyhow::Result<()> {
    let database = Database::connect(&config.database_url)
        .await
        .context("failed to connect to database")?;
    database.migrate().await?;

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

    // Bootstrap admin user if no users exist
    bootstrap_admin_user(&user_repo, config.admin_username, config.admin_password).await?;

    let state = AppState::new(
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
        reqwest::Client::new(),
        crate::infrastructure::foursquare::FOURSQUARE_SEARCH_URL.to_string(),
        config.foursquare_api_key,
        config.openrouter_api_key,
        config.openrouter_model,
    );

    let listener = TcpListener::bind(config.bind_address)
        .await
        .with_context(|| format!("failed to bind to {}", config.bind_address))?;

    let app: Router = app_router(state);

    info!(address = %config.bind_address, "starting HTTP server");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server terminated unexpectedly")?;

    info!("server shutdown complete");

    Ok(())
}

async fn bootstrap_admin_user(
    user_repo: &Arc<dyn UserRepository>,
    admin_username: Option<String>,
    admin_password: Option<String>,
) -> anyhow::Result<()> {
    // Check if any users exist
    let users_exist = user_repo
        .exists()
        .await
        .context("failed to check if users exist")?;

    if users_exist {
        // Users already exist, no need to bootstrap
        return Ok(());
    }

    // No users exist - we need to create the admin user
    let username = admin_username.ok_or_else(|| {
        anyhow::anyhow!(
            "No users exist in the database. Please provide BREWLOG_ADMIN_USERNAME \
             environment variable to create the admin user."
        )
    })?;

    let password = admin_password.ok_or_else(|| {
        anyhow::anyhow!(
            "No users exist in the database. Please provide BREWLOG_ADMIN_PASSWORD \
             environment variable to create the admin user."
        )
    })?;

    info!("No users found. Creating admin user '{}'...", username);

    let password_hash = hash_password(&password).context("failed to hash admin password")?;

    let admin_user = NewUser::new(username, password_hash);

    user_repo
        .insert(admin_user)
        .await
        .context("failed to create admin user")?;

    info!("Admin user created successfully");

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
