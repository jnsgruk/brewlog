use crate::domain::RepositoryError;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::TimelineEventId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::TimelineEventRepository;
use crate::domain::timeline::{
    NewTimelineEvent, TimelineBrewData, TimelineEvent, TimelineEventDetail, TimelineSortKey,
};
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
    async fn insert(&self, event: NewTimelineEvent) -> Result<TimelineEvent, RepositoryError> {
        let query = r"
            INSERT INTO timeline_events (entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json
        ";

        let details_json = serde_json::to_string(&event.details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })?;

        let tasting_notes_json = serde_json::to_string(&event.tasting_notes).map_err(|err| {
            RepositoryError::unexpected(format!(
                "failed to encode timeline event tasting notes: {err}"
            ))
        })?;

        let brew_data_json = event
            .brew_data
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to encode brew data: {err}"))
            })?;

        let record = sqlx::query_as::<_, TimelineEventRecord>(query)
            .bind(event.entity_type.as_str())
            .bind(event.entity_id)
            .bind(event.action)
            .bind(event.occurred_at)
            .bind(event.title)
            .bind(details_json)
            .bind(tasting_notes_json)
            .bind(event.slug)
            .bind(event.roaster_slug)
            .bind(brew_data_json)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        record.into_domain()
    }

    async fn update_by_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        event: NewTimelineEvent,
    ) -> Result<(), RepositoryError> {
        let details_json = serde_json::to_string(&event.details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })?;

        let tasting_notes_json = serde_json::to_string(&event.tasting_notes).map_err(|err| {
            RepositoryError::unexpected(format!(
                "failed to encode timeline event tasting notes: {err}"
            ))
        })?;

        let brew_data_json = event
            .brew_data
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|err| {
                RepositoryError::unexpected(format!("failed to encode brew data: {err}"))
            })?;

        sqlx::query(
            r"UPDATE timeline_events
              SET title = ?, details_json = ?, tasting_notes_json = ?,
                  slug = ?, roaster_slug = ?, brew_data_json = ?
              WHERE entity_type = ? AND entity_id = ?",
        )
        .bind(event.title)
        .bind(details_json)
        .bind(tasting_notes_json)
        .bind(event.slug)
        .bind(event.roaster_slug)
        .bind(brew_data_json)
        .bind(entity_type.as_str())
        .bind(entity_id)
        .execute(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(())
    }

    async fn delete_by_entity(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM timeline_events WHERE entity_type = ? AND entity_id = ?")
            .bind(entity_type.as_str())
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        Ok(())
    }

    async fn delete_all(&self) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM timeline_events")
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        Ok(())
    }

    async fn list(
        &self,
        request: &ListRequest<TimelineSortKey>,
    ) -> Result<Page<TimelineEvent>, RepositoryError> {
        let direction_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        let order_clause = format!("occurred_at {direction_sql}, id DESC");

        // All data is now denormalized in the timeline_events table - no JOINs needed
        let base_query = r"SELECT
            id, entity_type, entity_id, action, occurred_at, title,
            details_json, tasting_notes_json, slug, roaster_slug, brew_data_json
        FROM timeline_events";

        let count_query = "SELECT COUNT(*) FROM timeline_events";

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            base_query,
            count_query,
            &order_clause,
            None,
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
    action: String,
    occurred_at: DateTime<Utc>,
    title: String,
    details_json: Option<String>,
    tasting_notes_json: Option<String>,
    slug: Option<String>,
    roaster_slug: Option<String>,
    brew_data_json: Option<String>,
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

        let brew_data = match self.brew_data_json {
            Some(raw) if !raw.is_empty() => {
                Some(from_str::<TimelineBrewData>(&raw).map_err(|err| {
                    RepositoryError::unexpected(format!("failed to decode brew data: {err}"))
                })?)
            }
            _ => None,
        };

        let entity_type: EntityType = self.entity_type.parse().map_err(|()| {
            RepositoryError::unexpected(format!("unknown entity type: {}", self.entity_type))
        })?;

        Ok(TimelineEvent {
            id: TimelineEventId::from(self.id),
            entity_type,
            entity_id: self.entity_id,
            action: self.action,
            occurred_at: self.occurred_at,
            title: self.title,
            details,
            tasting_notes,
            slug: self.slug,
            roaster_slug: self.roaster_slug,
            brew_data,
        })
    }
}
