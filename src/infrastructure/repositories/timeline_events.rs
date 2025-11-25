use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::from_str;
use sqlx::{query_as, query_scalar};

use crate::domain::RepositoryError;
use crate::domain::ids::TimelineEventId;
use crate::domain::listing::{ListRequest, Page, PageSize, SortDirection};
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

        match request.page_size() {
            PageSize::All => {
                let query = format!(
                    "SELECT id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json \
                     FROM timeline_events \
                     ORDER BY {order_clause}"
                );

                let records = query_as::<_, TimelineEventRecord>(&query)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                let mut events = Vec::with_capacity(records.len());
                for record in records {
                    events.push(record.into_domain()?);
                }

                let total = events.len() as u64;
                let page_size = total.min(u64::from(u32::MAX)) as u32;
                Ok(Page::new(events, 1, page_size.max(1), total, true))
            }
            PageSize::Limited(page_size) => {
                let limit = page_size as i64;
                let mut page = request.page();
                let offset = ((page - 1) as i64).saturating_mul(limit);

                let query = format!(
                    "SELECT id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json \
                     FROM timeline_events \
                     ORDER BY {order_clause} \
                     LIMIT ? OFFSET ?"
                );

                let mut records = query_as::<_, TimelineEventRecord>(&query)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                let total: i64 = query_scalar("SELECT COUNT(*) FROM timeline_events")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                if page > 1 && records.is_empty() && total > 0 {
                    let last_page = ((total + limit - 1) / limit) as u32;
                    page = last_page.max(1);
                    let offset = ((page - 1) as i64).saturating_mul(limit);
                    records = query_as::<_, TimelineEventRecord>(&query)
                        .bind(limit)
                        .bind(offset)
                        .fetch_all(&self.pool)
                        .await
                        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
                }

                let mut events = Vec::with_capacity(records.len());
                for record in records {
                    events.push(record.into_domain()?);
                }

                Ok(Page::new(events, page, page_size, total as u64, false))
            }
        }
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
