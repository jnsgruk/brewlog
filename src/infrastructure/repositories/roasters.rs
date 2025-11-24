use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, query_as, query_scalar};

use crate::domain::RepositoryError;
use crate::domain::ids::generate_id;
use crate::domain::listing::{ListRequest, Page, PageSize, SortDirection};
use crate::domain::repositories::RoasterRepository;
use crate::domain::roasters::{Roaster, RoasterSortKey, UpdateRoaster};
use crate::domain::timeline::TimelineEventDetail;
use crate::infrastructure::database::DatabasePool;

type DbId = String;

#[derive(Clone)]
pub struct SqlRoasterRepository {
    pool: DatabasePool,
}

impl SqlRoasterRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn to_domain(record: RoasterRecord) -> Result<Roaster, RepositoryError> {
        let RoasterRecord {
            id,
            name,
            country,
            city,
            homepage,
            notes,
            created_at,
        } = record;

        Ok(Roaster {
            id,
            name,
            country,
            city,
            homepage,
            notes,
            created_at,
        })
    }
}

fn roaster_order_clause(request: &ListRequest<RoasterSortKey>) -> String {
    let dir_sql = match request.sort_direction() {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };

    match request.sort_key() {
        RoasterSortKey::CreatedAt => format!("created_at {dir_sql}, name ASC"),
        RoasterSortKey::Name => format!("LOWER(name) {dir_sql}, created_at DESC"),
        RoasterSortKey::Country => format!("LOWER(country) {dir_sql}, LOWER(name) ASC"),
        RoasterSortKey::City => {
            format!("LOWER(COALESCE(city, '')) {dir_sql}, LOWER(name) ASC")
        }
    }
}

#[async_trait]
impl RoasterRepository for SqlRoasterRepository {
    async fn insert(&self, roaster: Roaster) -> Result<Roaster, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let query = "INSERT INTO roasters (id, name, country, city, homepage, notes, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)";

        sqlx::query(query)
            .bind(&roaster.id)
            .bind(&roaster.name)
            .bind(&roaster.country)
            .bind(&roaster.city)
            .bind(&roaster.homepage)
            .bind(&roaster.notes)
            .bind(roaster.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let homepage_value = roaster
            .homepage
            .as_ref()
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| "—".to_string());

        let details = vec![
            TimelineEventDetail {
                label: "Country".to_string(),
                value: roaster.country.clone(),
            },
            TimelineEventDetail {
                label: "City".to_string(),
                value: roaster
                    .city
                    .as_ref()
                    .filter(|value| !value.is_empty())
                    .cloned()
                    .unwrap_or_else(|| "—".to_string()),
            },
            TimelineEventDetail {
                label: "Homepage".to_string(),
                value: homepage_value,
            },
        ];

        let details_json = serde_json::to_string(&details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })?;

        sqlx::query("INSERT INTO timeline_events (id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(generate_id())
            .bind("roaster")
            .bind(&roaster.id)
            .bind(roaster.created_at)
            .bind(&roaster.name)
            .bind(details_json)
            .bind(Option::<String>::None)
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(roaster)
    }

    async fn get(&self, id: String) -> Result<Roaster, RepositoryError> {
        let record = query_as::<_, RoasterRecord>(
            "SELECT id, name, country, city, homepage, notes, created_at FROM roasters WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Self::to_domain(record),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn list(
        &self,
        request: &ListRequest<RoasterSortKey>,
    ) -> Result<Page<Roaster>, RepositoryError> {
        let order_clause = roaster_order_clause(request);

        match request.page_size() {
            PageSize::All => {
                let query = format!(
                    "SELECT id, name, country, city, homepage, notes, created_at FROM roasters ORDER BY {}",
                    order_clause
                );

                let records = query_as::<_, RoasterRecord>(&query)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                let items = records
                    .into_iter()
                    .map(Self::to_domain)
                    .collect::<Result<Vec<_>, _>>()?;

                let total = items.len() as u64;
                let page_size = total.min(u64::from(u32::MAX)) as u32;
                Ok(Page::new(items, 1, page_size.max(1), total, true))
            }
            PageSize::Limited(page_size) => {
                let page_size_i64 = page_size as i64;
                let mut page = request.page();
                let offset = ((page - 1) as i64).saturating_mul(page_size_i64);

                let query = format!(
                    "SELECT id, name, country, city, homepage, notes, created_at FROM roasters ORDER BY {} LIMIT ? OFFSET ?",
                    order_clause
                );

                let mut records = query_as::<_, RoasterRecord>(&query)
                    .bind(page_size_i64)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                let total: i64 = query_scalar::<_, i64>("SELECT COUNT(*) FROM roasters")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                if page > 1 && records.is_empty() && total > 0 {
                    let last_page = ((total + page_size_i64 - 1) / page_size_i64) as u32;
                    page = last_page.max(1);
                    let offset = ((page - 1) as i64).saturating_mul(page_size_i64);
                    records = query_as::<_, RoasterRecord>(&query)
                        .bind(page_size_i64)
                        .bind(offset)
                        .fetch_all(&self.pool)
                        .await
                        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
                }

                let items = records
                    .into_iter()
                    .map(Self::to_domain)
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Page::new(items, page, page_size, total as u64, false))
            }
        }
    }

    async fn update(&self, id: String, changes: UpdateRoaster) -> Result<Roaster, RepositoryError> {
        let mut builder = QueryBuilder::new("UPDATE roasters SET ");
        let mut first = true;

        if let Some(name) = changes.name {
            if !first {
                builder.push(", ");
            }
            first = false;
            builder.push("name = ");
            builder.push_bind(name);
        }
        if let Some(country) = changes.country {
            if !first {
                builder.push(", ");
            }
            first = false;
            builder.push("country = ");
            builder.push_bind(country);
        }
        if let Some(city) = changes.city {
            if !first {
                builder.push(", ");
            }
            first = false;
            builder.push("city = ");
            builder.push_bind(city);
        }
        if let Some(homepage) = changes.homepage {
            if !first {
                builder.push(", ");
            }
            first = false;
            builder.push("homepage = ");
            builder.push_bind(homepage);
        }
        if let Some(notes) = changes.notes {
            if !first {
                builder.push(", ");
            }
            first = false;
            builder.push("notes = ");
            builder.push_bind(notes);
        }

        if first {
            return Err(RepositoryError::unexpected(
                "No fields provided for update".to_string(),
            ));
        }

        builder.push(" WHERE id = ");
        builder.push_bind(&id);

        let result = builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        self.get(id).await
    }

    async fn delete(&self, id: String) -> Result<(), RepositoryError> {
        let result = sqlx::query("DELETE FROM roasters WHERE id = ?")
            .bind(id)
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
struct RoasterRecord {
    id: DbId,
    name: String,
    country: String,
    city: Option<String>,
    homepage: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}
