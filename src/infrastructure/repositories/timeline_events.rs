use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::from_str;
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::repositories::TimelineEventRepository;
use crate::domain::timeline::{TimelineEvent, TimelineEventDetail};
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlTimelineEventRepository {
    pool: DatabasePool,
}

impl SqlTimelineEventRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimelineEventRepository for SqlTimelineEventRepository {
    async fn list_all(&self) -> Result<Vec<TimelineEvent>, RepositoryError> {
        let records = query_as::<_, TimelineEventRecord>(
            "SELECT id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json FROM timeline_events ORDER BY occurred_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let mut events = Vec::with_capacity(records.len());
        for record in records {
            events.push(record.into_domain()?);
        }

        Ok(events)
    }
}

#[derive(sqlx::FromRow)]
struct TimelineEventRecord {
    id: String,
    entity_type: String,
    entity_id: String,
    occurred_at: DateTime<Utc>,
    title: String,
    details_json: Option<String>,
    tasting_notes_json: Option<String>,
}

impl TimelineEventRecord {
    fn into_domain(self) -> Result<TimelineEvent, RepositoryError> {
        let details = match self.details_json {
            Some(raw) if !raw.is_empty() => {
                from_str::<Vec<TimelineEventDetail>>(&raw).map_err(|err| {
                    RepositoryError::unexpected(format!(
                        "failed to decode timeline event details: {err}"
                    ))
                })?
            }
            _ => Vec::new(),
        };

        let tasting_notes = match self.tasting_notes_json {
            Some(raw) if !raw.is_empty() => from_str::<Vec<String>>(&raw).map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to decode timeline event tasting notes: {err}"
                ))
            })?,
            _ => Vec::new(),
        };

        Ok(TimelineEvent {
            id: self.id,
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            occurred_at: self.occurred_at,
            title: self.title,
            details,
            tasting_notes,
        })
    }
}
