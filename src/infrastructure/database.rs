use anyhow::Context;
use sqlx::migrate::Migrator;

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
            use sqlx::sqlite::SqliteConnectOptions;
            use std::str::FromStr;

            let options = SqliteConnectOptions::from_str(database_url)
                .with_context(|| format!("invalid database url: {database_url}"))?
                .create_if_missing(true);

            PoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await
                .with_context(|| format!("failed to connect to database: {database_url}"))?
        };

        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .context("failed to enable foreign keys for sqlite")?;

        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&pool)
            .await
            .context("failed to enable WAL journal mode for sqlite")?;

        sqlx::query("PRAGMA synchronous = NORMAL;")
            .execute(&pool)
            .await
            .context("failed to set synchronous mode for sqlite")?;

        sqlx::query("PRAGMA cache_size = -8000;")
            .execute(&pool)
            .await
            .context("failed to set cache size for sqlite")?;

        sqlx::query("PRAGMA temp_store = MEMORY;")
            .execute(&pool)
            .await
            .context("failed to set temp_store for sqlite")?;

        sqlx::query("PRAGMA busy_timeout = 5000;")
            .execute(&pool)
            .await
            .context("failed to set busy_timeout for sqlite")?;

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
