use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::repositories::UserRepository;
use crate::domain::users::{User, UserId};
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlUserRepository {
    pool: DatabasePool,
}

impl SqlUserRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn to_domain(record: UserRecord) -> Result<User, RepositoryError> {
        let UserRecord {
            id,
            username,
            password_hash,
            created_at,
        } = record;

        Ok(User::new(id, username, password_hash, created_at))
    }
}

#[async_trait]
impl UserRepository for SqlUserRepository {
    async fn insert(&self, user: User) -> Result<User, RepositoryError> {
        let query = "INSERT INTO users (id, username, password_hash, created_at) VALUES (?, ?, ?, ?)";

        sqlx::query(query)
            .bind(&user.id)
            .bind(&user.username)
            .bind(&user.password_hash)
            .bind(&user.created_at)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err {
                    if db_err.is_unique_violation() {
                        return RepositoryError::conflict("user already exists");
                    }
                }
                RepositoryError::unexpected(err.to_string())
            })?;

        Ok(user)
    }

    async fn get(&self, id: UserId) -> Result<User, RepositoryError> {
        let query = "SELECT id, username, password_hash, created_at FROM users WHERE id = ?";

        let record = query_as::<_, UserRecord>(query)
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Self::to_domain(record)
    }

    async fn get_by_username(&self, username: &str) -> Result<User, RepositoryError> {
        let query = "SELECT id, username, password_hash, created_at FROM users WHERE username = ?";

        let record = query_as::<_, UserRecord>(query)
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Self::to_domain(record)
    }

    async fn exists(&self) -> Result<bool, RepositoryError> {
        let query = "SELECT COUNT(*) FROM users";

        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(count > 0)
    }
}

#[derive(sqlx::FromRow)]
struct UserRecord {
    id: UserId,
    username: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}
