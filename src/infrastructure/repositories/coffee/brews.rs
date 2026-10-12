use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, QueryBuilder, query_as};

use crate::domain::RepositoryError;
use crate::domain::brews::{
    Brew, BrewFilter, BrewSortKey, BrewWithDetails, NewBrew, QuickNote, UpdateBrew,
};
use crate::domain::ids::{BagId, BrewId, GearId};
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::BrewRepository;
use crate::infrastructure::database::{DatabasePool, DatabaseTransaction};
use crate::infrastructure::repositories::macros::push_update_field;

const BASE_SELECT: &str = r"
    SELECT
        br.id, br.bag_id, br.coffee_weight, br.grinder_id, br.grind_setting,
        br.brewer_id, br.filter_paper_id, br.water_volume, br.water_temp,
        br.quick_notes, br.brew_time,
        br.created_at, br.updated_at,
        r.name as roast_name, r.slug as roast_slug,
        rr.name as roaster_name, rr.slug as roaster_slug,
        (g_grinder.make || ' ' || g_grinder.model) as grinder_name,
        g_grinder.model as grinder_model,
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

#[derive(Clone)]
pub struct SqlBrewRepository {
    pool: DatabasePool,
}

impl SqlBrewRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn validate_dose(dose: f64) -> Result<(), RepositoryError> {
        if !dose.is_finite() || dose <= 0.0 {
            return Err(RepositoryError::conflict(
                "Coffee weight must be finite and greater than zero",
            ));
        }
        Ok(())
    }

    // The caller holds SQLite's write lock before reading stock or brew state.
    async fn deduct_stock(
        tx: &mut DatabaseTransaction<'_>,
        bag_id: BagId,
        dose: f64,
    ) -> Result<f64, RepositoryError> {
        let remaining: f64 =
            sqlx::query_scalar("SELECT remaining FROM bags WHERE id = ? AND closed = FALSE")
                .bind(bag_id.into_inner())
                .fetch_optional(&mut **tx)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?
                .ok_or_else(|| RepositoryError::conflict("Bag is closed or not found"))?;
        let deducted = dose.min(remaining.max(0.0));
        sqlx::query("UPDATE bags SET remaining = MAX(remaining - ?, 0), updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(deducted)
            .bind(bag_id.into_inner())
            .execute(&mut **tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        Ok(deducted)
    }

    async fn restore_stock(
        tx: &mut DatabaseTransaction<'_>,
        bag_id: BagId,
        credit: f64,
    ) -> Result<(), RepositoryError> {
        let result = sqlx::query("UPDATE bags SET remaining = remaining + ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(credit)
            .bind(bag_id.into_inner())
            .execute(&mut **tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(RepositoryError::conflict("Original bag not found"));
        }
        Ok(())
    }

    async fn reconcile_inventory(
        tx: &mut DatabaseTransaction<'_>,
        id: BrewId,
        changes: &UpdateBrew,
    ) -> Result<f64, RepositoryError> {
        let (old_bag, old_dose, mut credit): (i64, f64, f64) = sqlx::query_as(
            "SELECT bag_id, coffee_weight, inventory_deducted FROM brews WHERE id = ?",
        )
        .bind(id.into_inner())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .ok_or(RepositoryError::NotFound)?;
        let old_bag = BagId::new(old_bag);
        let new_bag = changes.bag_id.unwrap_or(old_bag);
        let new_dose = changes.coffee_weight.unwrap_or(old_dose);
        if new_bag != old_bag {
            Self::restore_stock(tx, old_bag, credit).await?;
            credit = Self::deduct_stock(tx, new_bag, new_dose).await?;
        } else if new_dose > old_dose {
            let closed: bool = sqlx::query_scalar("SELECT closed FROM bags WHERE id = ?")
                .bind(old_bag.into_inner())
                .fetch_one(&mut **tx)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
            if closed {
                return Err(RepositoryError::conflict(
                    "Cannot increase dose on a closed bag",
                ));
            }
            credit += Self::deduct_stock(tx, old_bag, new_dose - old_dose).await?;
        } else if new_dose < old_dose {
            // Closed bags still allow historical dose reductions.
            let new_credit = credit.min(new_dose);
            Self::restore_stock(tx, old_bag, credit - new_credit).await?;
            credit = new_credit;
        }
        Ok(credit)
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

    fn build_where_clause(filter: &BrewFilter) -> Option<String> {
        let mut conditions = Vec::new();

        // SAFETY: Direct interpolation is safe here because IDs are i64 from typed wrappers.
        if let Some(bag_id) = filter.bag_id {
            conditions.push(format!("br.bag_id = {}", bag_id.into_inner()));
        }
        if let Some(gear_id) = filter.gear_id {
            let id = gear_id.into_inner();
            conditions.push(format!(
                "(br.grinder_id = {id} OR br.brewer_id = {id} OR br.filter_paper_id = {id})"
            ));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

#[async_trait]
impl BrewRepository for SqlBrewRepository {
    async fn insert(&self, brew: NewBrew) -> Result<Brew, RepositoryError> {
        Self::validate_dose(brew.coffee_weight)?;
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        let deducted = Self::deduct_stock(&mut tx, brew.bag_id, brew.coffee_weight).await?;

        // Insert the brew
        let created_at = brew.created_at.unwrap_or_else(Utc::now);
        let insert_query = r"
            INSERT INTO brews (bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, brew_time, created_at, updated_at, inventory_deducted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, brew_time, created_at, updated_at
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
            .bind(brew.brew_time)
            .bind(created_at)
            .bind(created_at)
            .bind(deducted)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(record.into())
    }

    async fn get(&self, id: BrewId) -> Result<Brew, RepositoryError> {
        let query = r"
            SELECT id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, brew_time, created_at, updated_at
            FROM brews
            WHERE id = ?
        ";

        let record = query_as::<_, BrewRecord>(query)
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(record.into())
    }

    async fn get_with_details(&self, id: BrewId) -> Result<BrewWithDetails, RepositoryError> {
        let query = format!("{BASE_SELECT} WHERE br.id = ?");

        let record = query_as::<_, BrewWithDetailsRecord>(AssertSqlSafe(query))
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(record.into())
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
            |record: BrewWithDetailsRecord| Ok(record.into()),
        )
        .await
    }

    async fn update(&self, id: BrewId, changes: UpdateBrew) -> Result<Brew, RepositoryError> {
        if let Some(dose) = changes.coffee_weight {
            Self::validate_dose(dose)?;
        }
        let mut tx = self
            .pool
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        let credit = Self::reconcile_inventory(&mut tx, id, &changes).await?;
        let mut builder = QueryBuilder::new("UPDATE brews SET updated_at = CURRENT_TIMESTAMP");
        builder.push(", inventory_deducted = ");
        builder.push_bind(credit);
        let mut sep = true;

        push_update_field!(
            builder,
            sep,
            "bag_id",
            changes.bag_id.map(crate::domain::ids::BagId::into_inner)
        );
        push_update_field!(builder, sep, "coffee_weight", changes.coffee_weight);
        push_update_field!(
            builder,
            sep,
            "grinder_id",
            changes
                .grinder_id
                .map(crate::domain::ids::GearId::into_inner)
        );
        push_update_field!(builder, sep, "grind_setting", changes.grind_setting);
        push_update_field!(
            builder,
            sep,
            "brewer_id",
            changes
                .brewer_id
                .map(crate::domain::ids::GearId::into_inner)
        );
        push_update_field!(
            builder,
            sep,
            "filter_paper_id",
            changes
                .filter_paper_id
                .map(|value| value.map(crate::domain::ids::GearId::into_inner))
        );
        push_update_field!(builder, sep, "water_volume", changes.water_volume);
        push_update_field!(builder, sep, "water_temp", changes.water_temp);
        if let Some(ref notes) = changes.quick_notes {
            if sep {
                builder.push(", ");
            }
            sep = true;
            builder.push("quick_notes = ");
            builder.push_bind(Self::encode_quick_notes(notes));
        }
        push_update_field!(builder, sep, "brew_time", changes.brew_time);
        push_update_field!(builder, sep, "created_at", changes.created_at);
        let _ = sep;

        builder.push(" WHERE id = ");
        builder.push_bind(id.into_inner());
        builder.push(
            " RETURNING id, bag_id, coffee_weight, grinder_id, grind_setting, brewer_id, filter_paper_id, water_volume, water_temp, quick_notes, brew_time, created_at, updated_at",
        );

        let record = builder
            .build_query_as::<BrewRecord>()
            .fetch_optional(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
        Ok(record.into())
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
    brew_time: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<BrewRecord> for Brew {
    fn from(record: BrewRecord) -> Self {
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
            quick_notes: decode_quick_notes(record.quick_notes),
            brew_time: record.brew_time,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
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
    brew_time: Option<i32>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roast_name: String,
    roast_slug: String,
    roaster_name: String,
    roaster_slug: String,
    grinder_name: String,
    grinder_model: String,
    brewer_name: String,
    filter_paper_name: Option<String>,
}

impl From<BrewWithDetailsRecord> for BrewWithDetails {
    fn from(record: BrewWithDetailsRecord) -> Self {
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
                quick_notes: decode_quick_notes(record.quick_notes),
                brew_time: record.brew_time,
                created_at: record.created_at,
                updated_at: record.updated_at,
            },
            roast_name: record.roast_name,
            roaster_name: record.roaster_name,
            roast_slug: record.roast_slug,
            roaster_slug: record.roaster_slug,
            grinder_name: record.grinder_name,
            grinder_model: record.grinder_model,
            brewer_name: record.brewer_name,
            filter_paper_name: record.filter_paper_name,
        }
    }
}
