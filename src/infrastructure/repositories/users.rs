use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::ids::UserId;
use crate::domain::repositories::UserRepository;
use crate::domain::users::{NewUser, User};
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlUserRepository {
    pool: DatabasePool,
}

impl SqlUserRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn to_domain(record: UserRecord) -> User {
        let UserRecord {
            id,
            username,
            uuid,
            created_at,
        } = record;

        User::new(UserId::from(id), username, uuid, created_at)
    }
}

#[async_trait]
impl UserRepository for SqlUserRepository {
    async fn insert(&self, user: NewUser) -> Result<User, RepositoryError> {
        let query = "INSERT INTO users (username, uuid) VALUES (?, ?) RETURNING id, username, uuid, created_at";

        let record = sqlx::query_as::<_, UserRecord>(query)
            .bind(&user.username)
            .bind(&user.uuid)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return RepositoryError::conflict("user already exists");
                }
                RepositoryError::unexpected(err.to_string())
            })?;

        Ok(Self::to_domain(record))
    }

    async fn get(&self, id: UserId) -> Result<User, RepositoryError> {
        let query = "SELECT id, username, uuid, created_at FROM users WHERE id = ?";

        let record = query_as::<_, UserRecord>(query)
            .bind(i64::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn get_by_username(&self, username: &str) -> Result<User, RepositoryError> {
        let query = "SELECT id, username, uuid, created_at FROM users WHERE username = ?";

        let record = query_as::<_, UserRecord>(query)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn get_by_uuid(&self, uuid: &str) -> Result<User, RepositoryError> {
        let query = "SELECT id, username, uuid, created_at FROM users WHERE uuid = ?";

        let record = query_as::<_, UserRecord>(query)
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn exists(&self) -> Result<bool, RepositoryError> {
        let query = "SELECT COUNT(*) FROM users";

        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(count > 0)
    }

    async fn list_all(&self) -> Result<Vec<User>, RepositoryError> {
        let query = "SELECT id, username, uuid, created_at FROM users ORDER BY created_at ASC";

        let records = query_as::<_, UserRecord>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(records.into_iter().map(Self::to_domain).collect())
    }
}

#[derive(sqlx::FromRow)]
struct UserRecord {
    id: i64,
    username: String,
    uuid: String,
    created_at: DateTime<Utc>,
}
