use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::ids::{TokenId, UserId};
use crate::domain::repositories::TokenRepository;
use crate::domain::tokens::{NewToken, Token};
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlTokenRepository {
    pool: DatabasePool,
}

impl SqlTokenRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TokenRepository for SqlTokenRepository {
    async fn insert(&self, token: NewToken) -> Result<Token, RepositoryError> {
        let query = "INSERT INTO tokens (user_id, token_hash, name) VALUES (?, ?, ?) RETURNING id, user_id, token_hash, name, created_at, last_used_at, revoked_at";

        let NewToken {
            user_id,
            token_hash,
            name,
        } = token;

        let record = query_as::<_, TokenRecord>(query)
            .bind(i64::from(user_id))
            .bind(&token_hash)
            .bind(&name)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return RepositoryError::conflict("token already exists");
                }
                RepositoryError::unexpected(err.to_string())
            })?;

        Ok(record.into())
    }

    async fn get(&self, id: TokenId) -> Result<Token, RepositoryError> {
        let query = "SELECT id, user_id, token_hash, name, created_at, last_used_at, revoked_at FROM tokens WHERE id = ?";

        let record = query_as::<_, TokenRecord>(query)
            .bind(i64::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(record.into())
    }

    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Token, RepositoryError> {
        let query = "SELECT id, user_id, token_hash, name, created_at, last_used_at, revoked_at FROM tokens WHERE token_hash = ?";

        let record = query_as::<_, TokenRecord>(query)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(record.into())
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Token>, RepositoryError> {
        let query = "SELECT id, user_id, token_hash, name, created_at, last_used_at, revoked_at FROM tokens WHERE user_id = ? ORDER BY created_at DESC";

        let records = query_as::<_, TokenRecord>(query)
            .bind(i64::from(user_id))
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(records.into_iter().map(Into::into).collect())
    }

    async fn revoke(&self, id: TokenId) -> Result<Token, RepositoryError> {
        let query = "UPDATE tokens SET revoked_at = ? WHERE id = ? RETURNING id, user_id, token_hash, name, created_at, last_used_at, revoked_at";
        let now = Utc::now();

        let record = query_as::<_, TokenRecord>(query)
            .bind(now)
            .bind(i64::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Ok(record.into()),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn update_last_used(&self, id: TokenId) -> Result<(), RepositoryError> {
        let now = Utc::now();

        sqlx::query("UPDATE tokens SET last_used_at = ? WHERE id = ?")
            .bind(now)
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TokenRecord {
    id: i64,
    user_id: i64,
    token_hash: String,
    name: String,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

impl From<TokenRecord> for Token {
    fn from(record: TokenRecord) -> Self {
        Token::new(
            TokenId::from(record.id),
            UserId::from(record.user_id),
            record.token_hash,
            record.name,
            record.created_at,
            record.last_used_at,
            record.revoked_at,
        )
    }
}
