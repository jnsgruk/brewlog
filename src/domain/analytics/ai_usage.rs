use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domain::ids::{AiUsageId, UserId};

#[derive(Debug, Clone, Serialize)]
pub struct AiUsage {
    pub id: AiUsageId,
    pub user_id: UserId,
    pub model: String,
    pub endpoint: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewAiUsage {
    pub user_id: UserId,
    pub model: String,
    pub endpoint: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost: f64,
}

/// Aggregated usage totals for display.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AiUsageSummary {
    pub total_calls: i64,
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}
