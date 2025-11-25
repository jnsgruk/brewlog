use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite};

use crate::domain::sessions::{Session, SessionId};
use crate::domain::{RepositoryError, repositories::SessionRepository};

pub struct SqlSessionRepository {
    pool: Pool<Sqlite>,
}

impl SqlSessionRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for SqlSessionRepository {
    async fn insert(&self, session: Session) -> Result<Session, RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, session_token_hash, created_at, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.session_token_hash)
        .bind(session.created_at.to_rfc3339())
        .bind(session.expires_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::unexpected(format!("failed to insert session: {}", e)))?;

        Ok(session)
    }

    async fn get(&self, id: SessionId) -> Result<Session, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, session_token_hash, created_at, expires_at
            FROM sessions
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            _ => RepositoryError::unexpected(format!("failed to get session: {}", e)),
        })?;

        let created_at: String = row.try_get("created_at").map_err(|e| {
            RepositoryError::unexpected(format!("failed to parse created_at: {}", e))
        })?;
        let expires_at: String = row.try_get("expires_at").map_err(|e| {
            RepositoryError::unexpected(format!("failed to parse expires_at: {}", e))
        })?;

        Ok(Session {
            id: row.try_get("id").map_err(|e| {
                RepositoryError::unexpected(format!("failed to parse id: {}", e))
            })?,
            user_id: row.try_get("user_id").map_err(|e| {
                RepositoryError::unexpected(format!("failed to parse user_id: {}", e))
            })?,
            session_token_hash: row.try_get("session_token_hash").map_err(|e| {
                RepositoryError::unexpected(format!("failed to parse session_token_hash: {}", e))
            })?,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| {
                    RepositoryError::unexpected(format!("failed to parse created_at: {}", e))
                })?
                .with_timezone(&Utc),
            expires_at: DateTime::parse_from_rfc3339(&expires_at)
                .map_err(|e| {
                    RepositoryError::unexpected(format!("failed to parse expires_at: {}", e))
                })?
                .with_timezone(&Utc),
        })
    }

    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Session, RepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, session_token_hash, created_at, expires_at
            FROM sessions
            WHERE session_token_hash = ?
            "#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            _ => RepositoryError::unexpected(format!("failed to get session by token: {}", e)),
        })?;

        let created_at: String = row.try_get("created_at").map_err(|e| {
            RepositoryError::unexpected(format!("failed to parse created_at: {}", e))
        })?;
        let expires_at: String = row.try_get("expires_at").map_err(|e| {
            RepositoryError::unexpected(format!("failed to parse expires_at: {}", e))
        })?;

        Ok(Session {
            id: row.try_get("id").map_err(|e| {
                RepositoryError::unexpected(format!("failed to parse id: {}", e))
            })?,
            user_id: row.try_get("user_id").map_err(|e| {
                RepositoryError::unexpected(format!("failed to parse user_id: {}", e))
            })?,
            session_token_hash: row.try_get("session_token_hash").map_err(|e| {
                RepositoryError::unexpected(format!("failed to parse session_token_hash: {}", e))
            })?,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| {
                    RepositoryError::unexpected(format!("failed to parse created_at: {}", e))
                })?
                .with_timezone(&Utc),
            expires_at: DateTime::parse_from_rfc3339(&expires_at)
                .map_err(|e| {
                    RepositoryError::unexpected(format!("failed to parse expires_at: {}", e))
                })?
                .with_timezone(&Utc),
        })
    }

    async fn delete(&self, id: SessionId) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                RepositoryError::unexpected(format!("failed to delete session: {}", e))
            })?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<(), RepositoryError> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                RepositoryError::unexpected(format!("failed to delete expired sessions: {}", e))
            })?;

        Ok(())
    }
}
