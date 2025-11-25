use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::repositories::TokenRepository;
use crate::domain::tokens::{Token, TokenId};
use crate::domain::users::UserId;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlTokenRepository {
    pool: DatabasePool,
}

impl SqlTokenRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn to_domain(record: TokenRecord) -> Result<Token, RepositoryError> {
        let TokenRecord {
            id,
            user_id,
            token_hash,
            name,
            created_at,
            last_used_at,
            revoked_at,
        } = record;

        Ok(Token {
            id,
            user_id,
            token_hash,
            name,
            created_at,
            last_used_at,
            revoked_at,
        })
    }
}

#[async_trait]
impl TokenRepository for SqlTokenRepository {
    async fn insert(&self, token: Token) -> Result<Token, RepositoryError> {
        let query = "INSERT INTO tokens (id, user_id, token_hash, name, created_at, last_used_at, revoked_at) VALUES (?, ?, ?, ?, ?, ?, ?)";

        sqlx::query(query)
            .bind(&token.id)
            .bind(&token.user_id)
            .bind(&token.token_hash)
            .bind(&token.name)
            .bind(token.created_at)
            .bind(token.last_used_at)
            .bind(token.revoked_at)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return RepositoryError::conflict("token already exists");
                }
                RepositoryError::unexpected(err.to_string())
            })?;

        Ok(token)
    }

    async fn get(&self, id: TokenId) -> Result<Token, RepositoryError> {
        let query = "SELECT id, user_id, token_hash, name, created_at, last_used_at, revoked_at FROM tokens WHERE id = ?";

        let record = query_as::<_, TokenRecord>(query)
            .bind(&id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Self::to_domain(record)
    }

    async fn get_by_token_hash(&self, token_hash: &str) -> Result<Token, RepositoryError> {
        let query = "SELECT id, user_id, token_hash, name, created_at, last_used_at, revoked_at FROM tokens WHERE token_hash = ?";

        let record = query_as::<_, TokenRecord>(query)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Self::to_domain(record)
    }

    async fn list_by_user(&self, user_id: UserId) -> Result<Vec<Token>, RepositoryError> {
        let query = "SELECT id, user_id, token_hash, name, created_at, last_used_at, revoked_at FROM tokens WHERE user_id = ? ORDER BY created_at DESC";

        let records = query_as::<_, TokenRecord>(query)
            .bind(&user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        records.into_iter().map(Self::to_domain).collect()
    }

    async fn revoke(&self, id: TokenId) -> Result<Token, RepositoryError> {
        let query = "UPDATE tokens SET revoked_at = ? WHERE id = ?";
        let now = Utc::now();

        sqlx::query(query)
            .bind(&now)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        self.get(id).await
    }

    async fn update_last_used(&self, id: TokenId) -> Result<(), RepositoryError> {
        let query = "UPDATE tokens SET last_used_at = ? WHERE id = ?";
        let now = Utc::now();

        sqlx::query(query)
            .bind(&now)
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TokenRecord {
    id: TokenId,
    user_id: UserId,
    token_hash: String,
    name: String,
    created_at: DateTime<Utc>,
    last_used_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}
