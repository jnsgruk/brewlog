use crate::domain::RepositoryError;
use crate::domain::ids::TimelineEventId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::TimelineEventRepository;
use crate::domain::timeline::{TimelineEvent, TimelineEventDetail, TimelineSortKey};
use crate::infrastructure::database::DatabasePool;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::from_str;

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

        let order_clause = format!("t.occurred_at {direction_sql}, t.id DESC");
        let base_query = "SELECT 
            t.id, t.entity_type, t.entity_id, t.occurred_at, t.title, t.details_json, t.tasting_notes_json,
            CASE 
                WHEN t.entity_type = 'roaster' THEN r.slug 
                WHEN t.entity_type = 'roast' THEN rst.slug 
                ELSE NULL 
            END as slug,
            CASE 
                WHEN t.entity_type = 'roast' THEN rst_r.slug 
                ELSE NULL 
            END as roaster_slug
        FROM timeline_events t
        LEFT JOIN roasters r ON t.entity_type = 'roaster' AND t.entity_id = r.id
        LEFT JOIN roasts rst ON t.entity_type = 'roast' AND t.entity_id = rst.id
        LEFT JOIN roasters rst_r ON rst.roaster_id = rst_r.id";
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
    slug: Option<String>,
    roaster_slug: Option<String>,
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
            slug: self.slug,
            roaster_slug: self.roaster_slug,
        })
    }
}
