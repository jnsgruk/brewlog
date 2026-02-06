use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::brews::{Brew, BrewFilter, BrewSortKey, BrewWithDetails, NewBrew, QuickNote};
use crate::domain::ids::{BagId, BrewId, GearId};
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::BrewRepository;
use crate::infrastructure::database::DatabasePool;

const BASE_SELECT: &str = r"
    SELECT
        br.id, br.bag_id, br.coffee_weight, br.grinder_id, br.grind_setting,
        br.brewer_id, br.filter_paper_id, br.water_volume, br.water_temp,
        br.quick_notes,
        br.created_at, br.updated_at,
        r.name as roast_name, r.slug as roast_slug,
        rr.name as roaster_name, rr.slug as roaster_slug,
        (g_grinder.make || ' ' || g_grinder.model) as grinder_name,
        (g_brewer.make || ' ' || g_brewer.model) as brewer_name,
        (g_fp.make || ' ' || g_fp.model) as filter_paper_name
    FROM brews br
    JOIN bags b ON br.bag_id = b.id
    JOIN roasts r ON b.roast_id = r.id
    JOIN roasters rr ON r.roaster_id = rr.id
    JOIN gear g_grinder ON br.grinder_id = g_grinder.id
    JOIN gear g_brewer ON br.brewer_id = g_brewer.id
    LEFT JOIN gear g_fp ON br.filter_paper_id = g_fp.id
";

#[derive(Clone)]
pub struct SqlBrewRepository {
    pool: DatabasePool,
}

impl SqlBrewRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<BrewSortKey>) -> String {
        let dir_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        match request.sort_key() {
            BrewSortKey::CreatedAt => format!("br.created_at {dir_sql}, br.id DESC"),
            BrewSortKey::CoffeeWeight => format!("br.coffee_weight {dir_sql}, br.created_at DESC"),
            BrewSortKey::WaterVolume => format!("br.water_volume {dir_sql}, br.created_at DESC"),
        }
    }

    fn decode_quick_notes(raw: Option<String>) -> Vec<QuickNote> {
        match raw {
            Some(s) if !s.is_empty() => serde_json::from_str::<Vec<String>>(&s)
                .unwrap_or_default()
                .iter()
                .filter_map(|v| QuickNote::from_str_value(v))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn encode_quick_notes(notes: &[QuickNote]) -> Option<String> {
        if notes.is_empty() {
            None
        } else {
            let labels: Vec<&str> = notes.iter().map(|n| n.label()).collect();
            match serde_json::to_string(&labels) {
                Ok(json) => Some(json),
                Err(err) => {
                    tracing::warn!(error = %err, "failed to encode quick notes as JSON");
                    None
                }
            }
        }
    }

    fn to_domain(record: BrewRecord) -> Brew {
        Brew {
            id: BrewId::new(record.id),
            bag_id: BagId::new(record.bag_id),
            coffee_weight: record.coffee_weight,
            grinder_id: GearId::new(record.grinder_id),
            grind_setting: record.grind_setting,
            brewer_id: GearId::new(record.brewer_id),
            filter_paper_id: record.filter_paper_id.map(GearId::new),
            water_volume: record.water_volume,
            water_temp: record.water_temp,
            quick_notes: Self::decode_quick_notes(record.quick_notes),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }

    fn to_domain_with_details(record: BrewWithDetailsRecord) -> BrewWithDetails {
        BrewWithDetails {
            brew: Brew {
                id: BrewId::new(record.id),
                bag_id: BagId::new(record.bag_id),
                coffee_weight: record.coffee_weight,
                grinder_id: GearId::new(record.grinder_id),
                grind_setting: record.grind_setting,
                brewer_id: GearId::new(record.brewer_id),
                filter_paper_id: record.filter_paper_id.map(GearId::new),
                water_volume: record.water_volume,
                water_temp: record.water_temp,
                quick_notes: Self::decode_quick_notes(record.quick_notes),
                created_at: record.created_at,
                updated_at: record.updated_at,
            },
            roast_name: record.roast_name,
            roaster_name: record.roaster_name,
            roast_slug: record.roast_slug,
            roaster_slug: record.roaster_slug,
            grinder_name: record.grinder_name,
            brewer_name: record.brewer_name,
            filter_paper_name: record.filter_paper_name,
        }
    }

    fn build_where_clause(filter: &BrewFilter) -> Option<String> {
        // SAFETY: Direct interpolation is safe here because `bag_id` is an i64 from a typed wrapper.
        filter
            .bag_id
            .map(|bag_id| format!("br.bag_id = {}", bag_id.into_inner()))
    }
}

#[async_trait]
impl BrewRepository for SqlBrewRepository {
    async fn insert(&self, brew: NewBrew) -> Result<Brew, RepositoryError> {
        // Use a transaction to atomically:
        // 1. Deduct coffee_weight from bag's remaining
        // 2. Insert the brew
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        // Deduct coffee weight from bag's remaining amount
        let update_bag_query = r"
            UPDATE bags
            SET remaining = remaining - ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND remaining >= ? AND closed = FALSE
        ";

        let result = sqlx::query(update_bag_query)
            .bind(brew.coffee_weight)
            .bind(brew.bag_id.into_inner())
            .bind(brew.coffee_weight)
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::conflict(
                "Insufficient coffee remaining in bag or bag is closed",
            ));
        }

        // Insert the brew
        let insert_query = r"
            INSERT INTO brews (bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, created_at, updated_at
        ";

        let record = query_as::<_, BrewRecord>(insert_query)
            .bind(brew.bag_id.into_inner())
            .bind(brew.coffee_weight)
            .bind(brew.grinder_id.into_inner())
            .bind(brew.grind_setting)
            .bind(brew.brewer_id.into_inner())
            .bind(
                brew.filter_paper_id
                    .map(crate::domain::ids::GearId::into_inner),
            )
            .bind(brew.water_volume)
            .bind(brew.water_temp)
            .bind(Self::encode_quick_notes(&brew.quick_notes))
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(Self::to_domain(record))
    }

    async fn get(&self, id: BrewId) -> Result<Brew, RepositoryError> {
        let query = r"
            SELECT id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, created_at, updated_at
            FROM brews
            WHERE id = ?
        ";

        let record = query_as::<_, BrewRecord>(query)
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn get_with_details(&self, id: BrewId) -> Result<BrewWithDetails, RepositoryError> {
        let query = format!("{BASE_SELECT} WHERE br.id = ?");

        let record = query_as::<_, BrewWithDetailsRecord>(&query)
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain_with_details(record))
    }

    async fn list(
        &self,
        filter: BrewFilter,
        request: &ListRequest<BrewSortKey>,
        search: Option<&str>,
    ) -> Result<Page<BrewWithDetails>, RepositoryError> {
        use crate::infrastructure::repositories::pagination::SearchFilter;

        let order_clause = Self::order_clause(request);
        let where_clause = Self::build_where_clause(&filter);

        let base_query = match &where_clause {
            Some(w) => format!("{BASE_SELECT} WHERE {w}"),
            None => BASE_SELECT.to_string(),
        };

        let count_base = r"
            SELECT COUNT(*) FROM brews br
            JOIN bags b ON br.bag_id = b.id
            JOIN roasts r ON b.roast_id = r.id
            JOIN roasters rr ON r.roaster_id = rr.id
        ";

        let count_query = match &where_clause {
            Some(w) => format!("{count_base} WHERE {w}"),
            None => count_base.to_string(),
        };

        let sf = search.and_then(|t| SearchFilter::new(t, vec!["r.name", "rr.name"]));

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

    async fn delete(&self, id: BrewId) -> Result<(), RepositoryError> {
        let query = "DELETE FROM brews WHERE id = ?";

        let result = sqlx::query(query)
            .bind(id.into_inner())
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct BrewRecord {
    id: i64,
    bag_id: i64,
    coffee_weight: f64,
    grinder_id: i64,
    grind_setting: f64,
    brewer_id: i64,
    filter_paper_id: Option<i64>,
    water_volume: i32,
    water_temp: f64,
    quick_notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct BrewWithDetailsRecord {
    id: i64,
    bag_id: i64,
    coffee_weight: f64,
    grinder_id: i64,
    grind_setting: f64,
    brewer_id: i64,
    filter_paper_id: Option<i64>,
    water_volume: i32,
    water_temp: f64,
    quick_notes: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roast_name: String,
    roast_slug: String,
    roaster_name: String,
    roaster_slug: String,
    grinder_name: String,
    brewer_name: String,
    filter_paper_name: Option<String>,
}
