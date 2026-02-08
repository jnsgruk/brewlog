use async_trait::async_trait;
use chrono::Utc;
use sqlx::{query, query_as};

use crate::domain::ids::{SessionId, UserId};
use crate::domain::sessions::{NewSession, Session};
use crate::domain::{RepositoryError, repositories::SessionRepository};
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlSessionRepository {
    pool: DatabasePool,
}

impl SqlSessionRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn to_domain(record: SessionRecord) -> Session {
        let SessionRecord {
            id,
            user_id,
            session_token_hash,
            created_at,
            expires_at,
        } = record;

        Session::new(
            SessionId::from(id),
            UserId::from(user_id),
            session_token_hash,
            created_at,
            expires_at,
        )
    }
}

#[async_trait]
impl SessionRepository for SqlSessionRepository {
    async fn insert(&self, session: NewSession) -> Result<Session, RepositoryError> {
        let query = "INSERT INTO sessions (user_id, session_token_hash, created_at, expires_at) VALUES (?, ?, ?, ?) RETURNING id, user_id, session_token_hash, created_at, expires_at";

        let NewSession {
            user_id,
            session_token_hash,
            created_at,
            expires_at,
        } = session;

        let record = query_as::<_, SessionRecord>(query)
            .bind(i64::from(user_id))
            .bind(&session_token_hash)
            .bind(created_at)
            .bind(expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to insert session: {err}"))
            })?;

        Ok(Self::to_domain(record))
    }

    async fn get(&self, id: SessionId) -> Result<Session, RepositoryError> {
        let query = "SELECT id, user_id, session_token_hash, created_at, expires_at FROM sessions WHERE id = ?";

        let record = query_as::<_, SessionRecord>(query)
            .bind(i64::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(format!("failed to get session: {err}")))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Session, RepositoryError> {
        let query = "SELECT id, user_id, session_token_hash, created_at, expires_at FROM sessions WHERE session_token_hash = ?";

        let record = query_as::<_, SessionRecord>(query)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to get session by token: {err}"))
            })?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn delete(&self, id: SessionId) -> Result<(), RepositoryError> {
        query("DELETE FROM sessions WHERE id = ?")
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to delete session: {err}"))
            })?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<(), RepositoryError> {
        let now = Utc::now();
        query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to delete expired sessions: {err}"))
            })?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct SessionRecord {
    id: i64,
    user_id: i64,
    session_token_hash: String,
    created_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
}
