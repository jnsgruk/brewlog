use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as};

use crate::domain::RepositoryError;
use crate::domain::ids::{PasskeyCredentialId, UserId};
use crate::domain::passkey_credentials::{NewPasskeyCredential, PasskeyCredential};
use crate::domain::repositories::PasskeyCredentialRepository;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlPasskeyCredentialRepository {
    pool: DatabasePool,
}

impl SqlPasskeyCredentialRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasskeyCredentialRepository for SqlPasskeyCredentialRepository {
    async fn insert(
        &self,
        credential: NewPasskeyCredential,
    ) -> Result<PasskeyCredential, RepositoryError> {
        let sql = r"
            INSERT INTO passkey_credentials (user_id, credential_json, name)
            VALUES (?, ?, ?)
            RETURNING id, user_id, credential_json, name, created_at, last_used_at
        ";

        let record = query_as::<_, PasskeyCredentialRecord>(sql)
            .bind(i64::from(credential.user_id))
            .bind(&credential.credential_json)
            .bind(&credential.name)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to insert passkey credential: {err}"))
            })?;

        Ok(record.into())
    }

    async fn get(&self, id: PasskeyCredentialId) -> Result<PasskeyCredential, RepositoryError> {
        let sql = r"
            SELECT id, user_id, credential_json, name, created_at, last_used_at
            FROM passkey_credentials
            WHERE id = ?
        ";

        let record = query_as::<_, PasskeyCredentialRecord>(sql)
            .bind(i64::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => RepositoryError::NotFound,
                err => {
                    RepositoryError::unexpected(format!("failed to get passkey credential: {err}"))
                }
            })?;

        Ok(record.into())
    }

    async fn list_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Vec<PasskeyCredential>, RepositoryError> {
        let sql = r"
            SELECT id, user_id, credential_json, name, created_at, last_used_at
            FROM passkey_credentials
            WHERE user_id = ?
            ORDER BY created_at ASC
        ";

        let records = query_as::<_, PasskeyCredentialRecord>(sql)
            .bind(i64::from(user_id))
            .fetch_all(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to list passkey credentials: {err}"))
            })?;

        Ok(records.into_iter().map(Into::into).collect())
    }

    async fn list_all(&self) -> Result<Vec<PasskeyCredential>, RepositoryError> {
        let sql = r"
            SELECT id, user_id, credential_json, name, created_at, last_used_at
            FROM passkey_credentials
            ORDER BY created_at ASC
        ";

        let records = query_as::<_, PasskeyCredentialRecord>(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to list all passkey credentials: {err}"
                ))
            })?;

        Ok(records.into_iter().map(Into::into).collect())
    }

    async fn update_credential_json(
        &self,
        id: PasskeyCredentialId,
        credential_json: &str,
    ) -> Result<(), RepositoryError> {
        query("UPDATE passkey_credentials SET credential_json = ? WHERE id = ?")
            .bind(credential_json)
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to update passkey credential json: {err}"
                ))
            })?;

        Ok(())
    }

    async fn update_last_used(&self, id: PasskeyCredentialId) -> Result<(), RepositoryError> {
        let now = Utc::now();
        query("UPDATE passkey_credentials SET last_used_at = ? WHERE id = ?")
            .bind(now)
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to update passkey credential last_used_at: {err}"
                ))
            })?;

        Ok(())
    }

    async fn delete(&self, id: PasskeyCredentialId) -> Result<(), RepositoryError> {
        query("DELETE FROM passkey_credentials WHERE id = ?")
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to delete passkey credential: {err}"))
            })?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct PasskeyCredentialRecord {
    id: i64,
    user_id: i64,
    credential_json: String,
    name: String,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
}

impl From<PasskeyCredentialRecord> for PasskeyCredential {
    fn from(record: PasskeyCredentialRecord) -> Self {
        PasskeyCredential {
            id: PasskeyCredentialId::from(record.id),
            user_id: UserId::from(record.user_id),
            credential_json: record.credential_json,
            name: record.name,
            created_at: record.created_at,
            last_used_at: record.last_used_at,
        }
    }
}
