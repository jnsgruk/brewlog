use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, query, query_as};

use super::macros::push_update_field;
use crate::domain::RepositoryError;
use crate::domain::cafes::{Cafe, CafeSortKey, NewCafe, UpdateCafe};
use crate::domain::ids::CafeId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::CafeRepository;
use crate::domain::timeline::TimelineEventDetail;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlCafeRepository {
    pool: DatabasePool,
}

impl SqlCafeRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<CafeSortKey>) -> String {
        let dir_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        match request.sort_key() {
            CafeSortKey::CreatedAt => format!("created_at {dir_sql}, name ASC"),
            CafeSortKey::Name => format!("LOWER(name) {dir_sql}, created_at DESC"),
            CafeSortKey::City => format!("LOWER(city) {dir_sql}, LOWER(name) ASC"),
            CafeSortKey::Country => format!("LOWER(country) {dir_sql}, LOWER(name) ASC"),
        }
    }

    fn into_domain(record: CafeRecord) -> Cafe {
        let CafeRecord {
            id,
            name,
            slug,
            city,
            country,
            latitude,
            longitude,
            website,
            created_at,
            updated_at,
        } = record;

        Cafe {
            id: CafeId::from(id),
            name,
            slug,
            city,
            country,
            latitude,
            longitude,
            website,
            created_at,
            updated_at,
        }
    }

    fn details_for_cafe(cafe: &Cafe) -> Result<String, RepositoryError> {
        let website_value = cafe
            .website
            .as_ref()
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| "—".to_string());

        let details = vec![
            TimelineEventDetail {
                label: "City".to_string(),
                value: cafe.city.clone(),
            },
            TimelineEventDetail {
                label: "Country".to_string(),
                value: cafe.country.clone(),
            },
            TimelineEventDetail {
                label: "Website".to_string(),
                value: website_value,
            },
            TimelineEventDetail {
                label: "Position".to_string(),
                value: format!(
                    "https://www.google.com/maps?q={},{}",
                    cafe.latitude, cafe.longitude
                ),
            },
        ];

        serde_json::to_string(&details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })
    }
}

#[async_trait]
impl CafeRepository for SqlCafeRepository {
    async fn insert(&self, new_cafe: NewCafe) -> Result<Cafe, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let new_cafe = new_cafe.normalize();
        let slug = new_cafe.slug();
        let now = Utc::now();

        let record = query_as::<_, CafeRecord>(
                "INSERT INTO cafes (name, slug, city, country, latitude, longitude, website, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)\
                 RETURNING id, name, slug, city, country, latitude, longitude, website, created_at, updated_at",
            )
            .bind(&new_cafe.name)
            .bind(&slug)
            .bind(&new_cafe.city)
            .bind(&new_cafe.country)
            .bind(new_cafe.latitude)
            .bind(new_cafe.longitude)
            .bind(new_cafe.website.as_deref())
            .bind(now)
            .bind(now)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return RepositoryError::conflict(
                        "A cafe with this name and city already exists",
                    );
                }
                RepositoryError::unexpected(err.to_string())
            })?;

        let cafe = Self::into_domain(record);
        let details_json = Self::details_for_cafe(&cafe)?;

        query(
                "INSERT INTO timeline_events (entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind("cafe")
            .bind(i64::from(cafe.id))
            .bind("added")
            .bind(cafe.created_at)
            .bind(&cafe.name)
            .bind(details_json)
            .bind::<Option<&str>>(None)
            .bind(&cafe.slug)
            .bind::<Option<&str>>(None)
            .bind::<Option<&str>>(None)
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(cafe)
    }

    async fn get(&self, id: CafeId) -> Result<Cafe, RepositoryError> {
        let record = query_as::<_, CafeRecord>(
                "SELECT id, name, slug, city, country, latitude, longitude, website, created_at, updated_at FROM cafes WHERE id = ?",
            )
            .bind(i64::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Ok(Self::into_domain(record)),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn get_by_slug(&self, slug: &str) -> Result<Cafe, RepositoryError> {
        let record = query_as::<_, CafeRecord>(
                "SELECT id, name, slug, city, country, latitude, longitude, website, created_at, updated_at FROM cafes WHERE slug = ?",
            )
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match record {
            Some(record) => Ok(Self::into_domain(record)),
            None => Err(RepositoryError::NotFound),
        }
    }

    async fn list(
        &self,
        request: &ListRequest<CafeSortKey>,
        search: Option<&str>,
    ) -> Result<Page<Cafe>, RepositoryError> {
        use crate::infrastructure::repositories::pagination::SearchFilter;

        let order_clause = Self::order_clause(request);
        let base_query = "SELECT id, name, slug, city, country, latitude, longitude, website, created_at, updated_at FROM cafes";
        let count_query = "SELECT COUNT(*) FROM cafes";
        let sf = search.and_then(|t| SearchFilter::new(t, vec!["name", "city", "country"]));

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            base_query,
            count_query,
            &order_clause,
            sf.as_ref(),
            |record| Ok(Self::into_domain(record)),
        )
        .await
    }

    async fn update(&self, id: CafeId, changes: UpdateCafe) -> Result<Cafe, RepositoryError> {
        let mut builder = QueryBuilder::new("UPDATE cafes SET updated_at = CURRENT_TIMESTAMP");
        let mut sep = true;

        push_update_field!(builder, sep, "name", changes.name);
        push_update_field!(builder, sep, "city", changes.city);
        push_update_field!(builder, sep, "country", changes.country);
        push_update_field!(builder, sep, "latitude", changes.latitude);
        push_update_field!(builder, sep, "longitude", changes.longitude);
        push_update_field!(builder, sep, "website", changes.website);
        let _ = sep;

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

    async fn delete(&self, id: CafeId) -> Result<(), RepositoryError> {
        let result = query("DELETE FROM cafes WHERE id = ?")
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
struct CafeRecord {
    id: i64,
    name: String,
    slug: String,
    city: String,
    country: String,
    latitude: f64,
    longitude: f64,
    website: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
