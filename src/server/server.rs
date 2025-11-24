use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

use crate::domain::repositories::{RoastRepository, RoasterRepository, TimelineEventRepository};
use crate::infrastructure::database::Database;
use crate::infrastructure::repositories::roasters::SqlRoasterRepository;
use crate::infrastructure::repositories::roasts::SqlRoastRepository;
use crate::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;
use crate::server::routes::app_router;

pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub database_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub roaster_repo: Arc<dyn RoasterRepository>,
    pub roast_repo: Arc<dyn RoastRepository>,
    pub timeline_repo: Arc<dyn TimelineEventRepository>,
}

impl AppState {
    pub fn new(
        roaster_repo: Arc<dyn RoasterRepository>,
        roast_repo: Arc<dyn RoastRepository>,
        timeline_repo: Arc<dyn TimelineEventRepository>,
    ) -> Self {
        Self {
            roaster_repo,
            roast_repo,
            timeline_repo,
        }
    }
}

pub async fn serve(config: ServerConfig) -> anyhow::Result<()> {
    let database = Database::connect(&config.database_url)
        .await
        .context("failed to connect to database")?;
    database.migrate().await?;

    let roaster_repo = Arc::new(SqlRoasterRepository::new(database.clone_pool()));
    let roast_repo = Arc::new(SqlRoastRepository::new(database.clone_pool()));
    let timeline_repo = Arc::new(SqlTimelineEventRepository::new(database.clone_pool()));
    let state = AppState::new(roaster_repo, roast_repo, timeline_repo);

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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
