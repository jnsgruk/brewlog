use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, query, query_as};

use crate::domain::RepositoryError;
use crate::domain::ids::RoasterId;
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::RoasterRepository;
use crate::domain::roasters::{NewRoaster, Roaster, RoasterSortKey, UpdateRoaster};
use crate::domain::timeline::TimelineEventDetail;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlRoasterRepository {
    pool: DatabasePool,
}

impl SqlRoasterRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn sort_clause(request: &ListRequest<RoasterSortKey>) -> String {
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

    fn into_domain(record: RoasterRecord) -> Roaster {
        let RoasterRecord {
            id,
            name,
            country,
            city,
            homepage,
            notes,
            created_at,
        } = record;

        Roaster {
            id: RoasterId::from(id),
            name,
            country,
            city,
            homepage,
            notes,
            created_at,
        }
    }

    fn details_for_roaster(roaster: &Roaster) -> Result<String, RepositoryError> {
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

        serde_json::to_string(&details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })
    }
}

#[async_trait]
impl RoasterRepository for SqlRoasterRepository {
    async fn insert(&self, new_roaster: NewRoaster) -> Result<Roaster, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let new_roaster = new_roaster.normalize();
        let created_at = Utc::now();

        let record = query_as::<_, RoasterRecord>(
                "INSERT INTO roasters (name, country, city, homepage, notes, created_at) VALUES (?, ?, ?, ?, ?, ?)\
                 RETURNING id, name, country, city, homepage, notes, created_at",
            )
            .bind(&new_roaster.name)
            .bind(&new_roaster.country)
            .bind(new_roaster.city.as_deref())
            .bind(new_roaster.homepage.as_deref())
            .bind(new_roaster.notes.as_deref())
            .bind(created_at)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let roaster = Self::into_domain(record);
        let details_json = Self::details_for_roaster(&roaster)?;

        query(
                "INSERT INTO timeline_events (entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind("roaster")
            .bind(i64::from(roaster.id))
            .bind(roaster.created_at)
            .bind(&roaster.name)
            .bind(details_json)
            .bind::<Option<&str>>(None)
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(roaster)
    }

    async fn get(&self, id: RoasterId) -> Result<Roaster, RepositoryError> {
        let record = query_as::<_, RoasterRecord>(
                "SELECT id, name, country, city, homepage, notes, created_at FROM roasters WHERE id = ?",
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

    async fn list(
        &self,
        request: &ListRequest<RoasterSortKey>,
    ) -> Result<Page<Roaster>, RepositoryError> {
        let order_clause = Self::sort_clause(request);
        let base_query =
            "SELECT id, name, country, city, homepage, notes, created_at FROM roasters";
        let count_query = "SELECT COUNT(*) FROM roasters";

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            base_query,
            count_query,
            &order_clause,
            |record| Ok(Self::into_domain(record)),
        )
        .await
    }

    async fn update(
        &self,
        id: RoasterId,
        changes: UpdateRoaster,
    ) -> Result<Roaster, RepositoryError> {
        let mut builder = QueryBuilder::new("UPDATE roasters SET ");
        let mut wrote_field = false;

        if let Some(name) = changes.name {
            if wrote_field {
                builder.push(", ");
            }
            wrote_field = true;
            builder.push("name = ");
            builder.push_bind(name);
        }
        if let Some(country) = changes.country {
            if wrote_field {
                builder.push(", ");
            }
            wrote_field = true;
            builder.push("country = ");
            builder.push_bind(country);
        }
        if let Some(city) = changes.city {
            if wrote_field {
                builder.push(", ");
            }
            wrote_field = true;
            builder.push("city = ");
            builder.push_bind(city);
        }
        if let Some(homepage) = changes.homepage {
            if wrote_field {
                builder.push(", ");
            }
            wrote_field = true;
            builder.push("homepage = ");
            builder.push_bind(homepage);
        }
        if let Some(notes) = changes.notes {
            if wrote_field {
                builder.push(", ");
            }
            wrote_field = true;
            builder.push("notes = ");
            builder.push_bind(notes);
        }

        if !wrote_field {
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
    country: String,
    city: Option<String>,
    homepage: Option<String>,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}
