use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::from_str;
use crate::domain::RepositoryError;
use crate::domain::ids::TimelineEventId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::TimelineEventRepository;
use crate::domain::timeline::{TimelineEvent, TimelineEventDetail, TimelineSortKey};
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
    async fn list(
        &self,
        request: &ListRequest<TimelineSortKey>,
    ) -> Result<Page<TimelineEvent>, RepositoryError> {
        let direction_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        let order_clause = format!("occurred_at {direction_sql}, id DESC");
        let base_query = "SELECT id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json \
                     FROM timeline_events";
        let count_query = "SELECT COUNT(*) FROM timeline_events";

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            base_query,
            count_query,
            &order_clause,
            |record: TimelineEventRecord| record.into_domain(),
        )
        .await
    }
}

#[derive(sqlx::FromRow)]
struct TimelineEventRecord {
    id: i64,
    entity_type: String,
    entity_id: i64,
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
            id: TimelineEventId::from(self.id),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            occurred_at: self.occurred_at,
            title: self.title,
            details,
            tasting_notes,
        })
    }
}
