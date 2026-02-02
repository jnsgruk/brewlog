use crate::domain::RepositoryError;
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
            INSERT INTO timeline_events (entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json
        ";

        let details_json = serde_json::to_string(&event.details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })?;

        let tasting_notes_json = serde_json::to_string(&event.tasting_notes).map_err(|err| {
            RepositoryError::unexpected(format!(
                "failed to encode timeline event tasting notes: {err}"
            ))
        })?;

        let record = sqlx::query_as::<_, TimelineEventRecord>(query)
            .bind(event.entity_type)
            .bind(event.entity_id)
            .bind(event.action)
            .bind(event.occurred_at)
            .bind(event.title)
            .bind(details_json)
            .bind(tasting_notes_json)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        record.into_domain()
    }

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
            t.id, t.entity_type, t.entity_id, t.action, t.occurred_at, t.title, t.details_json, t.tasting_notes_json,
            CASE
                WHEN t.entity_type = 'roaster' THEN r.slug
                WHEN t.entity_type = 'roast' THEN rst.slug
                WHEN t.entity_type = 'bag' THEN b_r.slug
                WHEN t.entity_type = 'brew' THEN brew_roast.slug
                ELSE NULL
            END as slug,
            CASE
                WHEN t.entity_type = 'roast' THEN rst_r.slug
                WHEN t.entity_type = 'bag' THEN b_rr.slug
                WHEN t.entity_type = 'brew' THEN brew_roaster.slug
                ELSE NULL
            END as roaster_slug,
            brew.bag_id as brew_bag_id,
            brew.grinder_id as brew_grinder_id,
            brew.brewer_id as brew_brewer_id,
            brew.coffee_weight as brew_coffee_weight,
            brew.grind_setting as brew_grind_setting,
            brew.water_volume as brew_water_volume,
            brew.water_temp as brew_water_temp
        FROM timeline_events t
        LEFT JOIN roasters r ON t.entity_type = 'roaster' AND t.entity_id = r.id
        LEFT JOIN roasts rst ON t.entity_type = 'roast' AND t.entity_id = rst.id
        LEFT JOIN roasters rst_r ON rst.roaster_id = rst_r.id
        LEFT JOIN bags b ON t.entity_type = 'bag' AND t.entity_id = b.id
        LEFT JOIN roasts b_r ON b.roast_id = b_r.id
        LEFT JOIN roasters b_rr ON b_r.roaster_id = b_rr.id
        LEFT JOIN gear g ON t.entity_type = 'gear' AND t.entity_id = g.id
        LEFT JOIN brews brew ON t.entity_type = 'brew' AND t.entity_id = brew.id
        LEFT JOIN bags brew_bag ON brew.bag_id = brew_bag.id
        LEFT JOIN roasts brew_roast ON brew_bag.roast_id = brew_roast.id
        LEFT JOIN roasters brew_roaster ON brew_roast.roaster_id = brew_roaster.id";
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
    action: String,
    occurred_at: DateTime<Utc>,
    title: String,
    details_json: Option<String>,
    tasting_notes_json: Option<String>,
    slug: Option<String>,
    roaster_slug: Option<String>,
    // Brew-specific fields (only populated for brew events)
    brew_bag_id: Option<i64>,
    brew_grinder_id: Option<i64>,
    brew_brewer_id: Option<i64>,
    brew_coffee_weight: Option<f64>,
    brew_grind_setting: Option<f64>,
    brew_water_volume: Option<i32>,
    brew_water_temp: Option<f64>,
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

        // Build brew_data if all required fields are present
        let brew_data = match (
            self.brew_bag_id,
            self.brew_grinder_id,
            self.brew_brewer_id,
            self.brew_coffee_weight,
            self.brew_grind_setting,
            self.brew_water_volume,
            self.brew_water_temp,
        ) {
            (
                Some(bag_id),
                Some(grinder_id),
                Some(brewer_id),
                Some(coffee_weight),
                Some(grind_setting),
                Some(water_volume),
                Some(water_temp),
            ) => Some(TimelineBrewData {
                bag_id,
                grinder_id,
                brewer_id,
                coffee_weight,
                grind_setting,
                water_volume,
                water_temp,
            }),
            _ => None,
        };

        Ok(TimelineEvent {
            id: TimelineEventId::from(self.id),
            entity_type: self.entity_type,
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
