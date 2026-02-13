use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, query, query_as};

use crate::domain::RepositoryError;
use crate::domain::ids::RoasterId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::RoasterRepository;
use crate::domain::roasters::{NewRoaster, Roaster, RoasterSortKey, UpdateRoaster};
use crate::infrastructure::database::DatabasePool;
use crate::infrastructure::repositories::macros::push_update_field;

#[derive(Clone)]
pub struct SqlRoasterRepository {
    pool: DatabasePool,
}

impl SqlRoasterRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<RoasterSortKey>) -> String {
        let dir_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        match request.sort_key() {
            RoasterSortKey::CreatedAt => format!("created_at {dir_sql}, name ASC"),
            RoasterSortKey::Name => format!("LOWER(name) {dir_sql}, created_at DESC"),
            RoasterSortKey::Country => format!("LOWER(country) {dir_sql}, LOWER(name) ASC"),
            RoasterSortKey::City => format!("LOWER(COALESCE(city, '')) {dir_sql}, LOWER(name) ASC"),
        }
    }
}

#[async_trait]
impl RoasterRepository for SqlRoasterRepository {
    async fn insert(&self, new_roaster: NewRoaster) -> Result<Roaster, RepositoryError> {
        let new_roaster = new_roaster.normalize();
        let slug = new_roaster.slug();
        let created_at = new_roaster.created_at.unwrap_or_else(Utc::now);

        let record = query_as::<_, RoasterRecord>(
                "INSERT INTO roasters (name, slug, country, city, homepage, created_at) VALUES (?, ?, ?, ?, ?, ?)\
                 RETURNING id, name, slug, country, city, homepage, created_at",
            )
            .bind(&new_roaster.name)
            .bind(&slug)
            .bind(&new_roaster.country)
            .bind(new_roaster.city.as_deref())
            .bind(new_roaster.homepage.as_deref())
            .bind(created_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return RepositoryError::conflict(
                        "A roaster with this name and city already exists",
                    );
                }
                RepositoryError::unexpected(err.to_string())
            })?;

        Ok(record.into())
    }

    async fn get(&self, id: RoasterId) -> Result<Roaster, RepositoryError> {
        let record = query_as::<_, RoasterRecord>(
            "SELECT id, name, slug, country, city, homepage, created_at FROM roasters WHERE id = ?",
        )
        .bind(i64::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Ok(record.into()),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Roaster, RepositoryError> {
        let record = query_as::<_, RoasterRecord>(
                "SELECT id, name, slug, country, city, homepage, created_at FROM roasters WHERE slug = ?",
            )
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Ok(record.into()),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn list(
        &self,
        request: &ListRequest<RoasterSortKey>,
        search: Option<&str>,
    ) -> Result<Page<Roaster>, RepositoryError> {
        use crate::infrastructure::repositories::pagination::SearchFilter;

        let order_clause = Self::order_clause(request);
        let base_query = "SELECT id, name, slug, country, city, homepage, created_at FROM roasters";
        let count_query = "SELECT COUNT(*) FROM roasters";
        let sf =
            search.and_then(|t| SearchFilter::new(t, vec!["name", "country", "COALESCE(city,'')"]));

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            base_query,
            count_query,
            &order_clause,
            sf.as_ref(),
            |record: RoasterRecord| Ok(record.into()),
        )
        .await
    }

    async fn update(
        &self,
        id: RoasterId,
        changes: UpdateRoaster,
    ) -> Result<Roaster, RepositoryError> {
        let mut builder = QueryBuilder::new("UPDATE roasters SET ");
        let mut sep = false;

        push_update_field!(builder, sep, "name", changes.name);
        push_update_field!(builder, sep, "country", changes.country);
        push_update_field!(builder, sep, "city", changes.city);
        push_update_field!(builder, sep, "homepage", changes.homepage);
        push_update_field!(builder, sep, "created_at", changes.created_at);

        if !sep {
            return Err(RepositoryError::unexpected(
                "No fields provided for update".to_string(),
            ));
        }

        builder.push(" WHERE id = ");
        builder.push_bind(i64::from(id));

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

    async fn delete(&self, id: RoasterId) -> Result<(), RepositoryError> {
        let result = query("DELETE FROM roasters WHERE id = ?")
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
struct RoasterRecord {
    id: i64,
    name: String,
    slug: String,
    country: String,
    city: Option<String>,
    homepage: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<RoasterRecord> for Roaster {
    fn from(record: RoasterRecord) -> Self {
        Roaster {
            id: RoasterId::from(record.id),
            name: record.name,
            slug: record.slug,
            country: record.country,
            city: record.city,
            homepage: record.homepage,
            created_at: record.created_at,
        }
    }
}
