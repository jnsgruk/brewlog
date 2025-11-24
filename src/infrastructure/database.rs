use anyhow::Context;
use sqlx::migrate::Migrator;

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("features `sqlite` and `postgres` cannot both be enabled");

#[cfg(not(any(feature = "sqlite", feature = "postgres")))]
compile_error!("enable either the `sqlite` or `postgres` feature");

#[cfg(feature = "sqlite")]
pub type DatabasePool = sqlx::SqlitePool;
#[cfg(feature = "sqlite")]
type PoolOptions = sqlx::sqlite::SqlitePoolOptions;
#[cfg(feature = "sqlite")]
pub type DatabaseTransaction<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

#[cfg(feature = "postgres")]
pub type DatabasePool = sqlx::PgPool;
#[cfg(feature = "postgres")]
type PoolOptions = sqlx::postgres::PgPoolOptions;
#[cfg(feature = "postgres")]
pub type DatabaseTransaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

pub struct Database {
    pool: DatabasePool,
}

impl Database {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        #[cfg(feature = "sqlite")]
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

        #[cfg(feature = "postgres")]
        let pool = PoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .with_context(|| format!("failed to connect to database: {database_url}"))?;

        #[cfg(feature = "sqlite")]
        {
            sqlx::query("PRAGMA foreign_keys = ON;")
                .execute(&pool)
                .await
                .context("failed to enable foreign keys for sqlite")?;
        }

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
