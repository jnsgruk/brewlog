use std::sync::Arc;

use brewlog::domain::repositories::{RoastRepository, RoasterRepository, TimelineEventRepository};
use brewlog::infrastructure::database::Database;
use brewlog::infrastructure::repositories::roasters::SqlRoasterRepository;
use brewlog::infrastructure::repositories::roasts::SqlRoastRepository;
use brewlog::infrastructure::repositories::timeline_events::SqlTimelineEventRepository;
use brewlog::server::routes::app_router;
use brewlog::server::server::AppState;
use tokio::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub roaster_repo: Arc<dyn RoasterRepository>,
    pub roast_repo: Arc<dyn RoastRepository>,
    #[allow(dead_code)]
    pub timeline_repo: Arc<dyn TimelineEventRepository>,
}

impl TestApp {
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.address, path)
    }
}

pub async fn spawn_app() -> TestApp {
    // Use in-memory SQLite database for testing
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory database");

    // Run migrations
    database.migrate().await.expect("Failed to migrate database");

    // Create repositories
    let roaster_repo = Arc::new(SqlRoasterRepository::new(database.clone_pool()));
    let roast_repo = Arc::new(SqlRoastRepository::new(database.clone_pool()));
    let timeline_repo = Arc::new(SqlTimelineEventRepository::new(database.clone_pool()));

    // Create application state
    let state = AppState::new(
        roaster_repo.clone(),
        roast_repo.clone(),
        timeline_repo.clone(),
    );

    // Create router
    let app = app_router(state);

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to random port");

    let local_addr = listener.local_addr().expect("Failed to get local address");
    let address = format!("http://{}", local_addr);

    // Spawn the server in a background task
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Server failed to start");
    });

    TestApp {
        address,
        roaster_repo,
        roast_repo,
        timeline_repo,
    }
}
