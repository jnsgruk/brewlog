use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::ids::{RegistrationTokenId, UserId};
use crate::domain::registration_tokens::{NewRegistrationToken, RegistrationToken};
use crate::domain::repositories::RegistrationTokenRepository;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlRegistrationTokenRepository {
    pool: DatabasePool,
}

impl SqlRegistrationTokenRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RegistrationTokenRepository for SqlRegistrationTokenRepository {
    async fn insert(
        &self,
        token: NewRegistrationToken,
    ) -> Result<RegistrationToken, RepositoryError> {
        let sql = r"
            INSERT INTO registration_tokens (token_hash, created_at, expires_at)
            VALUES (?, ?, ?)
            RETURNING id, token_hash, created_at, expires_at, used_at, used_by_user_id
        ";

        let record = query_as::<_, RegistrationTokenRecord>(sql)
            .bind(&token.token_hash)
            .bind(token.created_at)
            .bind(token.expires_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to insert registration token: {err}"))
            })?;

        Ok(record.into())
    }

    async fn get_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<RegistrationToken, RepositoryError> {
        let sql = r"
            SELECT id, token_hash, created_at, expires_at, used_at, used_by_user_id
            FROM registration_tokens
            WHERE token_hash = ?
        ";

        let record = query_as::<_, RegistrationTokenRecord>(sql)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to get registration token: {err}"))
            })?
            .ok_or(RepositoryError::NotFound)?;

        Ok(record.into())
    }

    async fn mark_used(
        &self,
        id: RegistrationTokenId,
        user_id: UserId,
    ) -> Result<(), RepositoryError> {
        let now = Utc::now();
        let sql = "UPDATE registration_tokens SET used_at = ?, used_by_user_id = ? WHERE id = ?";

        sqlx::query(sql)
            .bind(now)
            .bind(i64::from(user_id))
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to mark registration token as used: {err}"
                ))
            })?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RegistrationTokenRecord {
    id: i64,
    token_hash: String,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    used_by_user_id: Option<i64>,
}

impl From<RegistrationTokenRecord> for RegistrationToken {
    fn from(record: RegistrationTokenRecord) -> Self {
        RegistrationToken {
            id: RegistrationTokenId::from(record.id),
            token_hash: record.token_hash,
            created_at: record.created_at,
            expires_at: record.expires_at,
            used_at: record.used_at,
            used_by_user_id: record.used_by_user_id.map(UserId::from),
        }
    }
}
