use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::ai_usage::{AiUsage, AiUsageSummary, NewAiUsage};
use crate::domain::ids::{AiUsageId, UserId};
use crate::domain::repositories::AiUsageRepository;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlAiUsageRepository {
    pool: DatabasePool,
}

impl SqlAiUsageRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AiUsageRepository for SqlAiUsageRepository {
    async fn insert(&self, usage: NewAiUsage) -> Result<AiUsage, RepositoryError> {
        let query = r"
            INSERT INTO ai_usage (user_id, model, endpoint, prompt_tokens, completion_tokens, total_tokens, cost)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id, user_id, model, endpoint, prompt_tokens, completion_tokens, total_tokens, cost, created_at
        ";

        let record = query_as::<_, AiUsageRecord>(query)
            .bind(i64::from(usage.user_id))
            .bind(&usage.model)
            .bind(&usage.endpoint)
            .bind(usage.prompt_tokens)
            .bind(usage.completion_tokens)
            .bind(usage.total_tokens)
            .bind(usage.cost)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(record.into())
    }

    async fn summary_for_user(&self, user_id: UserId) -> Result<AiUsageSummary, RepositoryError> {
        let query = r"
            SELECT
                COALESCE(COUNT(*), 0) as total_calls,
                COALESCE(SUM(prompt_tokens), 0) as total_prompt_tokens,
                COALESCE(SUM(completion_tokens), 0) as total_completion_tokens,
                COALESCE(SUM(total_tokens), 0) as total_tokens,
                COALESCE(SUM(cost), 0.0) as total_cost
            FROM ai_usage
            WHERE user_id = ?
        ";

        let record = query_as::<_, AiUsageSummaryRecord>(query)
            .bind(i64::from(user_id))
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(AiUsageSummary {
            total_calls: record.total_calls,
            total_prompt_tokens: record.total_prompt_tokens,
            total_completion_tokens: record.total_completion_tokens,
            total_tokens: record.total_tokens,
            total_cost: record.total_cost,
        })
    }
}

#[derive(sqlx::FromRow)]
struct AiUsageRecord {
    id: i64,
    user_id: i64,
    model: String,
    endpoint: String,
    prompt_tokens: i64,
    completion_tokens: i64,
    total_tokens: i64,
    cost: f64,
    created_at: DateTime<Utc>,
}

impl From<AiUsageRecord> for AiUsage {
    fn from(record: AiUsageRecord) -> Self {
        AiUsage {
            id: AiUsageId::from(record.id),
            user_id: UserId::from(record.user_id),
            model: record.model,
            endpoint: record.endpoint,
            prompt_tokens: record.prompt_tokens,
            completion_tokens: record.completion_tokens,
            total_tokens: record.total_tokens,
            cost: record.cost,
            created_at: record.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
struct AiUsageSummaryRecord {
    total_calls: i64,
    total_prompt_tokens: i64,
    total_completion_tokens: i64,
    total_tokens: i64,
    total_cost: f64,
}
