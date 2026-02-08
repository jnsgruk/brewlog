use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, query_as};

use crate::domain::RepositoryError;
use crate::domain::gear::{Gear, GearCategory, GearFilter, GearSortKey, NewGear, UpdateGear};
use crate::domain::ids::GearId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::GearRepository;
use crate::infrastructure::database::DatabasePool;
use crate::infrastructure::repositories::macros::push_update_field;

#[derive(Clone)]
pub struct SqlGearRepository {
    pool: DatabasePool,
}

impl SqlGearRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<GearSortKey>) -> String {
        let dir_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        match request.sort_key() {
            GearSortKey::Make => format!("LOWER(make) {dir_sql}, created_at DESC"),
            GearSortKey::Model => format!("LOWER(model) {dir_sql}, created_at DESC"),
            GearSortKey::Category => format!("category {dir_sql}, LOWER(make) ASC"),
            GearSortKey::CreatedAt => format!("created_at {dir_sql}, id DESC"),
        }
    }

    fn to_domain(record: GearRecord) -> Result<Gear, RepositoryError> {
        let category = GearCategory::from_str(&record.category).map_err(|()| {
            RepositoryError::unexpected(format!("invalid category: {}", record.category))
        })?;

        Ok(Gear {
            id: GearId::new(record.id),
            category,
            make: record.make,
            model: record.model,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }

    fn build_where_clause(filter: &GearFilter) -> Option<&'static str> {
        filter.category.as_ref().map(|category| match category {
            GearCategory::Grinder => "category = 'grinder'",
            GearCategory::Brewer => "category = 'brewer'",
            GearCategory::FilterPaper => "category = 'filter_paper'",
        })
    }
}

#[async_trait]
impl GearRepository for SqlGearRepository {
    async fn insert(&self, gear: NewGear) -> Result<Gear, RepositoryError> {
        let created_at = gear.created_at.unwrap_or_else(Utc::now);
        let query = r"
            INSERT INTO gear (category, make, model, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, category, make, model, created_at, updated_at
        ";

        let record = query_as::<_, GearRecord>(query)
            .bind(gear.category.as_str())
            .bind(&gear.make)
            .bind(&gear.model)
            .bind(created_at)
            .bind(created_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Self::to_domain(record)
    }

    async fn get(&self, id: GearId) -> Result<Gear, RepositoryError> {
        let query = r"
            SELECT id, category, make, model, created_at, updated_at
            FROM gear
            WHERE id = ?
        ";

        let record = query_as::<_, GearRecord>(query)
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Self::to_domain(record)
    }

    async fn list(
        &self,
        filter: GearFilter,
        request: &ListRequest<GearSortKey>,
        search: Option<&str>,
    ) -> Result<Page<Gear>, RepositoryError> {
        use crate::infrastructure::repositories::pagination::SearchFilter;

        let order_clause = Self::order_clause(request);
        let where_clause = Self::build_where_clause(&filter);

        let base_query = match &where_clause {
            Some(w) => format!(
                "SELECT id, category, make, model, created_at, updated_at FROM gear WHERE {w}"
            ),
            None => {
                "SELECT id, category, make, model, created_at, updated_at FROM gear".to_string()
            }
        };

        let count_query = match &where_clause {
            Some(w) => format!("SELECT COUNT(*) FROM gear WHERE {w}"),
            None => "SELECT COUNT(*) FROM gear".to_string(),
        };

        let sf = search.and_then(|t| SearchFilter::new(t, vec!["make", "model"]));

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            &base_query,
            &count_query,
            &order_clause,
            sf.as_ref(),
            Self::to_domain,
        )
        .await
    }

    async fn update(&self, id: GearId, changes: UpdateGear) -> Result<Gear, RepositoryError> {
        let mut builder = QueryBuilder::new("UPDATE gear SET updated_at = CURRENT_TIMESTAMP");
        let mut sep = true; // Already have updated_at

        push_update_field!(builder, sep, "make", changes.make);
        push_update_field!(builder, sep, "model", changes.model);
        push_update_field!(builder, sep, "created_at", changes.created_at);
        let _ = sep; // Suppress unused_assignments warning

        builder.push(" WHERE id = ");
        builder.push_bind(id.into_inner());
        builder.push(" RETURNING id, category, make, model, created_at, updated_at");

        let record = builder
            .build_query_as::<GearRecord>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Self::to_domain(record)
    }

    async fn delete(&self, id: GearId) -> Result<(), RepositoryError> {
        let query = "DELETE FROM gear WHERE id = ?";

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
struct GearRecord {
    id: i64,
    category: String,
    make: String,
    model: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
