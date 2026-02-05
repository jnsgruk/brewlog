use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{query, query_as};

use crate::domain::RepositoryError;
use crate::domain::cups::{Cup, CupFilter, CupSortKey, CupWithDetails, NewCup};
use crate::domain::ids::{CafeId, CupId, RoastId};
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::CupRepository;
use crate::domain::timeline::TimelineEventDetail;
use crate::infrastructure::database::DatabasePool;

const BASE_SELECT: &str = r"
    SELECT
        c.id, c.roast_id, c.cafe_id,
        c.created_at, c.updated_at,
        r.name as roast_name, r.slug as roast_slug,
        rr.name as roaster_name, rr.slug as roaster_slug,
        ca.name as cafe_name, ca.slug as cafe_slug
    FROM cups c
    JOIN roasts r ON c.roast_id = r.id
    JOIN roasters rr ON r.roaster_id = rr.id
    JOIN cafes ca ON c.cafe_id = ca.id
";

#[derive(Clone)]
pub struct SqlCupRepository {
    pool: DatabasePool,
}

impl SqlCupRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<CupSortKey>) -> String {
        let dir_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        match request.sort_key() {
            CupSortKey::CreatedAt => format!("c.created_at {dir_sql}, c.id DESC"),
            CupSortKey::CafeName => {
                format!("LOWER(ca.name) {dir_sql}, c.created_at DESC")
            }
            CupSortKey::RoastName => {
                format!("LOWER(r.name) {dir_sql}, c.created_at DESC")
            }
        }
    }

    fn to_domain(record: CupRecord) -> Cup {
        Cup {
            id: CupId::new(record.id),
            roast_id: RoastId::new(record.roast_id),
            cafe_id: CafeId::new(record.cafe_id),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }

    fn to_domain_with_details(record: CupWithDetailsRecord) -> CupWithDetails {
        CupWithDetails {
            cup: Cup {
                id: CupId::new(record.id),
                roast_id: RoastId::new(record.roast_id),
                cafe_id: CafeId::new(record.cafe_id),
                created_at: record.created_at,
                updated_at: record.updated_at,
            },
            roast_name: record.roast_name,
            roaster_name: record.roaster_name,
            roast_slug: record.roast_slug,
            roaster_slug: record.roaster_slug,
            cafe_name: record.cafe_name,
            cafe_slug: record.cafe_slug,
        }
    }

    fn build_where_clause(filter: &CupFilter) -> Option<String> {
        let mut conditions = Vec::new();

        // SAFETY: Direct interpolation is safe here because IDs are i64 from typed wrappers.
        if let Some(cafe_id) = filter.cafe_id {
            conditions.push(format!("c.cafe_id = {}", cafe_id.into_inner()));
        }
        if let Some(roast_id) = filter.roast_id {
            conditions.push(format!("c.roast_id = {}", roast_id.into_inner()));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }

    fn details_for_cup(cup_with_details: &CupWithDetails) -> Result<String, RepositoryError> {
        let details = vec![
            TimelineEventDetail {
                label: "Coffee".to_string(),
                value: cup_with_details.roast_name.clone(),
            },
            TimelineEventDetail {
                label: "Roaster".to_string(),
                value: cup_with_details.roaster_name.clone(),
            },
            TimelineEventDetail {
                label: "Cafe".to_string(),
                value: cup_with_details.cafe_name.clone(),
            },
        ];

        serde_json::to_string(&details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })
    }
}

#[async_trait]
impl CupRepository for SqlCupRepository {
    async fn insert(&self, new_cup: NewCup) -> Result<Cup, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let record = query_as::<_, CupRecord>(
            "INSERT INTO cups (roast_id, cafe_id) VALUES (?, ?) \
             RETURNING id, roast_id, cafe_id, created_at, updated_at",
        )
        .bind(new_cup.roast_id.into_inner())
        .bind(new_cup.cafe_id.into_inner())
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let cup = Self::to_domain(record);

        // Fetch enriched details for the timeline event
        let details_record =
            query_as::<_, CupWithDetailsRecord>(&format!("{BASE_SELECT} WHERE c.id = ?"))
                .bind(cup.id.into_inner())
                .fetch_one(&mut *tx)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let cup_with_details = Self::to_domain_with_details(details_record);
        let details_json = Self::details_for_cup(&cup_with_details)?;

        let title = cup_with_details.roast_name.clone();

        query(
            "INSERT INTO timeline_events (entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("cup")
        .bind(i64::from(cup.id))
        .bind("added")
        .bind(cup.created_at)
        .bind(&title)
        .bind(details_json)
        .bind::<Option<&str>>(None)
        .bind(&cup_with_details.roast_slug)
        .bind(&cup_with_details.roaster_slug)
        .bind::<Option<&str>>(None)
        .execute(&mut *tx)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(cup)
    }

    async fn get(&self, id: CupId) -> Result<Cup, RepositoryError> {
        let record = query_as::<_, CupRecord>(
            "SELECT id, roast_id, cafe_id, created_at, updated_at FROM cups WHERE id = ?",
        )
        .bind(i64::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Ok(Self::to_domain(record)),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn get_with_details(&self, id: CupId) -> Result<CupWithDetails, RepositoryError> {
        let query = format!("{BASE_SELECT} WHERE c.id = ?");

        let record = query_as::<_, CupWithDetailsRecord>(&query)
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain_with_details(record))
    }

    async fn list(
        &self,
        filter: CupFilter,
        request: &ListRequest<CupSortKey>,
        search: Option<&str>,
    ) -> Result<Page<CupWithDetails>, RepositoryError> {
        use crate::infrastructure::repositories::pagination::SearchFilter;

        let order_clause = Self::order_clause(request);
        let where_clause = Self::build_where_clause(&filter);

        let base_query = match &where_clause {
            Some(w) => format!("{BASE_SELECT} WHERE {w}"),
            None => BASE_SELECT.to_string(),
        };

        let count_base = r"
            SELECT COUNT(*) FROM cups c
            JOIN roasts r ON c.roast_id = r.id
            JOIN roasters rr ON r.roaster_id = rr.id
            JOIN cafes ca ON c.cafe_id = ca.id
        ";

        let count_query = match &where_clause {
            Some(w) => format!("{count_base} WHERE {w}"),
            None => count_base.to_string(),
        };

        let sf = search.and_then(|t| SearchFilter::new(t, vec!["r.name", "rr.name", "ca.name"]));

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            &base_query,
            &count_query,
            &order_clause,
            sf.as_ref(),
            |record| Ok(Self::to_domain_with_details(record)),
        )
        .await
    }

    async fn delete(&self, id: CupId) -> Result<(), RepositoryError> {
        let result = query("DELETE FROM cups WHERE id = ?")
            .bind(i64::from(id))
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CupRecord {
    id: i64,
    roast_id: i64,
    cafe_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct CupWithDetailsRecord {
    id: i64,
    roast_id: i64,
    cafe_id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roast_name: String,
    roast_slug: String,
    roaster_name: String,
    roaster_slug: String,
    cafe_name: String,
    cafe_slug: String,
}
