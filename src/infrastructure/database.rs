use anyhow::Context;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};

pub type DatabasePool = sqlx::SqlitePool;
type PoolOptions = sqlx::sqlite::SqlitePoolOptions;
pub type DatabaseTransaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;
pub type DatabaseRow = sqlx::sqlite::SqliteRow;
pub type DatabaseDriver = sqlx::Sqlite;

pub struct Database {
    pool: DatabasePool,
}

impl Database {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = {
            use std::str::FromStr;

            let options = SqliteConnectOptions::from_str(database_url)
                .with_context(|| format!("invalid database url: {database_url}"))?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .foreign_keys(true)
                .pragma("cache_size", "-8000")
                .pragma("temp_store", "MEMORY")
                .busy_timeout(std::time::Duration::from_secs(5));

            PoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .with_context(|| format!("failed to connect to database: {database_url}"))?
        };

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    pub fn clone_pool(&self) -> DatabasePool {
        self.pool.clone()
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
        MIGRATOR
            .run(&self.pool)
            .await
            .context("database migration failed")
    }
}
