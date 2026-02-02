use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{QueryBuilder, query_as};

use super::macros::push_update_field;
use crate::domain::RepositoryError;
use crate::domain::bags::{Bag, BagFilter, BagSortKey, BagWithRoast, NewBag, UpdateBag};
use crate::domain::ids::{BagId, RoastId};
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::BagRepository;
use crate::infrastructure::database::DatabasePool;

const BASE_SELECT: &str = r#"
    SELECT 
        b.id, b.roast_id, b.roast_date, b.amount, b.remaining, b.closed, b.finished_at, b.created_at, b.updated_at,
        r.name as roast_name, r.slug as roast_slug,
        rr.name as roaster_name, rr.slug as roaster_slug
    FROM bags b
    JOIN roasts r ON b.roast_id = r.id
    JOIN roasters rr ON r.roaster_id = rr.id
"#;

#[derive(Clone)]
pub struct SqlBagRepository {
    pool: DatabasePool,
}

impl SqlBagRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<BagSortKey>) -> String {
        let sort_column = match request.sort_key {
            BagSortKey::RoastDate => "b.roast_date",
            BagSortKey::CreatedAt => "b.created_at",
            BagSortKey::Roaster => "rr.name",
            BagSortKey::Roast => "r.name",
            BagSortKey::FinishedAt => "b.finished_at",
        };

        let direction = match request.sort_direction {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        format!("{} {}", sort_column, direction)
    }

    fn to_domain(record: BagRecord) -> Bag {
        Bag {
            id: BagId::new(record.id),
            roast_id: RoastId::new(record.roast_id),
            roast_date: record.roast_date,
            amount: record.amount,
            remaining: record.remaining,
            closed: record.closed,
            finished_at: record.finished_at,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }

    fn to_domain_with_roast(record: BagWithRoastRecord) -> BagWithRoast {
        BagWithRoast {
            bag: Bag {
                id: BagId::new(record.id),
                roast_id: RoastId::new(record.roast_id),
                roast_date: record.roast_date,
                amount: record.amount,
                remaining: record.remaining,
                closed: record.closed,
                finished_at: record.finished_at,
                created_at: record.created_at,
                updated_at: record.updated_at,
            },
            roast_name: record.roast_name,
            roaster_name: record.roaster_name,
            roast_slug: record.roast_slug,
            roaster_slug: record.roaster_slug,
        }
    }

    fn build_where_clause(filter: &BagFilter) -> Option<String> {
        let mut conditions = Vec::new();

        if let Some(closed) = filter.closed {
            conditions.push(format!(
                "b.closed = {}",
                if closed { "TRUE" } else { "FALSE" }
            ));
        }

        if let Some(roast_id) = filter.roast_id {
            conditions.push(format!("b.roast_id = {}", roast_id.into_inner()));
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

#[async_trait]
impl BagRepository for SqlBagRepository {
    async fn insert(&self, bag: NewBag) -> Result<Bag, RepositoryError> {
        let query = r#"
            INSERT INTO bags (roast_id, roast_date, amount, remaining)
            VALUES (?, ?, ?, ?)
            RETURNING id, roast_id, roast_date, amount, remaining, closed, finished_at, created_at, updated_at
        "#;

        let record = query_as::<_, BagRecord>(query)
            .bind(bag.roast_id.into_inner())
            .bind(bag.roast_date)
            .bind(bag.amount)
            .bind(bag.amount) // remaining starts as amount
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(Self::to_domain(record))
    }

    async fn get(&self, id: BagId) -> Result<Bag, RepositoryError> {
        let query = r#"
            SELECT id, roast_id, roast_date, amount, remaining, closed, finished_at, created_at, updated_at
            FROM bags
            WHERE id = ?
        "#;

        let record = query_as::<_, BagRecord>(query)
            .bind(id.into_inner())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn list(
        &self,
        filter: BagFilter,
        request: &ListRequest<BagSortKey>,
    ) -> Result<Page<BagWithRoast>, RepositoryError> {
        let order_clause = Self::order_clause(request);

        // Build WHERE clause from filter
        let where_clause = Self::build_where_clause(&filter);

        let base_query = match &where_clause {
            Some(w) => format!("{} WHERE {}", BASE_SELECT, w),
            None => BASE_SELECT.to_string(),
        };

        let count_query = match &where_clause {
            Some(w) => format!("SELECT COUNT(*) FROM bags b WHERE {}", w),
            None => "SELECT COUNT(*) FROM bags".to_string(),
        };

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            &base_query,
            &count_query,
            &order_clause,
            |record| Ok(Self::to_domain_with_roast(record)),
        )
        .await
    }

    async fn update(&self, id: BagId, changes: UpdateBag) -> Result<Bag, RepositoryError> {
        let mut builder = QueryBuilder::new("UPDATE bags SET updated_at = CURRENT_TIMESTAMP");
        let mut sep = true; // Already have updated_at

        push_update_field!(builder, sep, "remaining", changes.remaining);
        push_update_field!(builder, sep, "closed", changes.closed);
        push_update_field!(builder, sep, "finished_at", changes.finished_at);
        let _ = sep; // Suppress unused_assignments warning from macro

        builder.push(" WHERE id = ");
        builder.push_bind(id.into_inner());
        builder.push(" RETURNING id, roast_id, roast_date, amount, remaining, closed, finished_at, created_at, updated_at");

        let record = builder
            .build_query_as::<BagRecord>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .ok_or(RepositoryError::NotFound)?;

        Ok(Self::to_domain(record))
    }

    async fn delete(&self, id: BagId) -> Result<(), RepositoryError> {
        let query = "DELETE FROM bags WHERE id = ?";

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
struct BagRecord {
    id: i64,
    roast_id: i64,
    roast_date: Option<NaiveDate>,
    amount: f64,
    remaining: f64,
    closed: bool,
    finished_at: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct BagWithRoastRecord {
    id: i64,
    roast_id: i64,
    roast_date: Option<NaiveDate>,
    amount: f64,
    remaining: f64,
    closed: bool,
    finished_at: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    roast_name: String,
    roast_slug: String,
    roaster_name: String,
    roaster_slug: String,
}
