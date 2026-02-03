use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{from_str, to_string};
use sqlx::{Error as SqlxError, QueryBuilder, query, query_as};

use super::macros::push_update_field;
use crate::domain::RepositoryError;
use crate::domain::ids::{RoastId, RoasterId};
use crate::domain::listing::{ListRequest, Page, SortDirection};
use crate::domain::repositories::RoastRepository;
use crate::domain::roasts::{NewRoast, Roast, RoastSortKey, RoastWithRoaster, UpdateRoast};
use crate::domain::timeline::TimelineEventDetail;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlRoastRepository {
    pool: DatabasePool,
}

impl SqlRoastRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn order_clause(request: &ListRequest<RoastSortKey>) -> String {
        let dir_sql = match request.sort_direction() {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        };

        match request.sort_key() {
            RoastSortKey::CreatedAt => format!("r.created_at {dir_sql}, LOWER(r.name) ASC"),
            RoastSortKey::Name => format!("LOWER(r.name) {dir_sql}, r.created_at DESC"),
            RoastSortKey::Roaster => format!("LOWER(ro.name) {dir_sql}, r.created_at DESC"),
            RoastSortKey::Origin => {
                format!("LOWER(COALESCE(r.origin, '')) {dir_sql}, r.created_at DESC")
            }
            RoastSortKey::Producer => {
                format!("LOWER(COALESCE(r.producer, '')) {dir_sql}, r.created_at DESC")
            }
        }
    }

    fn encode_notes(notes: &[String]) -> Result<Option<String>, RepositoryError> {
        if notes.is_empty() {
            Ok(None)
        } else {
            to_string(notes).map(Some).map_err(|err| {
                RepositoryError::unexpected(format!("failed to encode tasting notes: {err}"))
            })
        }
    }
}

#[allow(clippy::too_many_lines)] // Repository impl has many methods
#[async_trait]
impl RoastRepository for SqlRoastRepository {
    async fn insert(&self, new_roast: NewRoast) -> Result<Roast, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let slug = new_roast.slug();
        let NewRoast {
            roaster_id,
            name,
            origin,
            region,
            producer,
            tasting_notes,
            process,
        } = new_roast;

        let origin_value = if origin.trim().is_empty() {
            None
        } else {
            Some(origin)
        };
        let region_value = if region.trim().is_empty() {
            None
        } else {
            Some(region)
        };
        let producer_value = if producer.trim().is_empty() {
            None
        } else {
            Some(producer)
        };
        let process_value = if process.trim().is_empty() {
            None
        } else {
            Some(process)
        };

        let created_at = Utc::now();
        let notes_json = Self::encode_notes(&tasting_notes)?;

        let record = query_as::<_, RoastRecord>(
                "INSERT INTO roasts (roaster_id, name, slug, origin, region, producer, process, tasting_notes, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)\
                 RETURNING id, roaster_id, name, slug, origin, region, producer, process, tasting_notes, created_at",
            )
            .bind(i64::from(roaster_id))
            .bind(&name)
            .bind(&slug)
            .bind(origin_value.as_deref())
            .bind(region_value.as_deref())
            .bind(producer_value.as_deref())
            .bind(process_value.as_deref())
            .bind(notes_json.as_deref())
            .bind(created_at)
            .fetch_one(&mut *tx)
            .await
            .map_err(|err| {
                if let sqlx::Error::Database(db_err) = &err
                    && db_err.is_unique_violation()
                {
                    return RepositoryError::conflict(
                        "A roast with this name already exists for this roaster",
                    );
                }
                map_insert_error(err, "unknown roaster reference")
            })?;

        let roast = record.into_roast()?;

        // Fetch roaster info for timeline event
        let roaster_info: Option<(String, String)> =
            query_as("SELECT name, slug FROM roasters WHERE id = ?")
                .bind(i64::from(roast.roaster_id))
                .fetch_optional(&mut *tx)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let (roaster_label, roaster_slug) = roaster_info.map_or_else(
            || ("Unknown roaster".to_string(), None),
            |(name, slug)| (name, Some(slug)),
        );

        let details = vec![
            TimelineEventDetail {
                label: "Roaster".to_string(),
                value: roaster_label.clone(),
            },
            TimelineEventDetail {
                label: "Origin".to_string(),
                value: roast.origin.clone().unwrap_or_else(|| "—".to_string()),
            },
            TimelineEventDetail {
                label: "Region".to_string(),
                value: roast.region.clone().unwrap_or_else(|| "—".to_string()),
            },
            TimelineEventDetail {
                label: "Producer".to_string(),
                value: roast.producer.clone().unwrap_or_else(|| "—".to_string()),
            },
            TimelineEventDetail {
                label: "Process".to_string(),
                value: roast.process.clone().unwrap_or_else(|| "—".to_string()),
            },
        ];

        let details_json = to_string(&details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })?;

        let tasting_notes_json = if roast.tasting_notes.is_empty() {
            None
        } else {
            Some(to_string(&roast.tasting_notes).map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to encode timeline event tasting notes: {err}"
                ))
            })?)
        };

        query(
                "INSERT INTO timeline_events (entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind("roast")
            .bind(i64::from(roast.id))
            .bind("added")
            .bind(roast.created_at)
            .bind(&roast.name)
            .bind(details_json)
            .bind(tasting_notes_json.as_deref())
            .bind(&roast.slug) // slug = roast's own slug
            .bind(roaster_slug.as_deref()) // roaster_slug from the roaster
            .bind::<Option<&str>>(None) // brew_data_json not applicable
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(roast)
    }

    async fn get(&self, id: RoastId) -> Result<Roast, RepositoryError> {
        query_as::<_, RoastRecord>(
                "SELECT id, roaster_id, name, slug, origin, region, producer, process, tasting_notes, created_at FROM roasts WHERE id = ?",
            )
            .bind(i64::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .map(RoastRecord::into_roast)
            .transpose()?
            .ok_or(RepositoryError::NotFound)
    }

    async fn get_with_roaster(&self, id: RoastId) -> Result<RoastWithRoaster, RepositoryError> {
        query_as::<_, RoastWithRoasterRecord>(
            "SELECT r.id, r.roaster_id, r.name, r.slug, r.origin, r.region, r.producer, r.process, r.tasting_notes, r.created_at, ro.name AS roaster_name, ro.slug AS roaster_slug \
             FROM roasts r \
             JOIN roasters ro ON ro.id = r.roaster_id \
             WHERE r.id = ?",
        )
        .bind(i64::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .map(RoastWithRoasterRecord::into_with_roaster)
        .transpose()?
        .ok_or(RepositoryError::NotFound)
    }

    async fn get_by_slug(
        &self,
        roaster_id: RoasterId,
        slug: &str,
    ) -> Result<Roast, RepositoryError> {
        query_as::<_, RoastRecord>(
                "SELECT id, roaster_id, name, slug, origin, region, producer, process, tasting_notes, created_at FROM roasts WHERE roaster_id = ? AND slug = ?",
            )
            .bind(i64::from(roaster_id))
            .bind(slug)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?
            .map(RoastRecord::into_roast)
            .transpose()?
            .ok_or(RepositoryError::NotFound)
    }

    async fn list(
        &self,
        request: &ListRequest<RoastSortKey>,
        search: Option<&str>,
    ) -> Result<Page<RoastWithRoaster>, RepositoryError> {
        use crate::infrastructure::repositories::pagination::SearchFilter;

        let order_clause = Self::order_clause(request);
        let base_query = "SELECT r.id, r.roaster_id, r.name, r.slug, r.origin, r.region, r.producer, r.process, r.tasting_notes, r.created_at, ro.name AS roaster_name, ro.slug AS roaster_slug \n                     FROM roasts r \n                     JOIN roasters ro ON ro.id = r.roaster_id";
        let count_query = "SELECT COUNT(*) FROM roasts r JOIN roasters ro ON ro.id = r.roaster_id";
        let sf = search.and_then(|t| {
            SearchFilter::new(
                t,
                vec![
                    "r.name",
                    "ro.name",
                    "COALESCE(r.origin,'')",
                    "COALESCE(r.producer,'')",
                ],
            )
        });

        crate::infrastructure::repositories::pagination::paginate(
            &self.pool,
            request,
            base_query,
            count_query,
            &order_clause,
            sf.as_ref(),
            |record: RoastWithRoasterRecord| record.into_with_roaster(),
        )
        .await
    }

    async fn list_by_roaster(
        &self,
        roaster_id: RoasterId,
    ) -> Result<Vec<RoastWithRoaster>, RepositoryError> {
        let records = query_as::<_, RoastWithRoasterRecord>(
                "SELECT r.id, r.roaster_id, r.name, r.slug, r.origin, r.region, r.producer, r.process, r.tasting_notes, r.created_at, ro.name AS roaster_name, ro.slug AS roaster_slug \n             FROM roasts r \n             JOIN roasters ro ON ro.id = r.roaster_id \n             WHERE r.roaster_id = ? \n             ORDER BY r.created_at DESC",
            )
            .bind(i64::from(roaster_id))
            .fetch_all(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        records
            .into_iter()
            .map(RoastWithRoasterRecord::into_with_roaster)
            .collect()
    }

    async fn update(&self, id: RoastId, changes: UpdateRoast) -> Result<Roast, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let mut builder = QueryBuilder::new("UPDATE roasts SET ");
        let mut sep = false;

        // Handle roaster_id specially due to type conversion
        if let Some(roaster_id) = changes.roaster_id {
            sep = true;
            builder.push("roaster_id = ");
            builder.push_bind(i64::from(roaster_id));
        }

        push_update_field!(builder, sep, "name", changes.name);
        push_update_field!(builder, sep, "origin", changes.origin);
        push_update_field!(builder, sep, "region", changes.region);
        push_update_field!(builder, sep, "producer", changes.producer);
        push_update_field!(builder, sep, "process", changes.process);

        // Handle tasting_notes specially due to JSON encoding
        if let Some(tasting_notes) = changes.tasting_notes {
            let notes_json = Self::encode_notes(&tasting_notes)?;
            if sep {
                builder.push(", ");
            }
            sep = true;
            builder.push("tasting_notes = ");
            builder.push_bind(notes_json);
        }

        if !sep {
            return Err(RepositoryError::unexpected(
                "No fields provided for update".to_string(),
            ));
        }

        builder.push(" WHERE id = ");
        builder.push_bind(i64::from(id));

        let result = builder
            .build()
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        self.get(id).await
    }

    async fn delete(&self, id: RoastId) -> Result<(), RepositoryError> {
        let result = query("DELETE FROM roasts WHERE id = ?")
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

fn map_insert_error(err: SqlxError, message: &'static str) -> RepositoryError {
    if let SqlxError::Database(db_err) = &err
        && db_err.code().as_deref() == Some("787")
    {
        return RepositoryError::conflict(message);
    }

    RepositoryError::unexpected(err.to_string())
}

#[derive(sqlx::FromRow)]
struct RoastRecord {
    id: i64,
    roaster_id: i64,
    name: String,
    slug: String,
    origin: Option<String>,
    region: Option<String>,
    producer: Option<String>,
    process: Option<String>,
    tasting_notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl RoastRecord {
    fn into_roast(self) -> Result<Roast, RepositoryError> {
        let tasting_notes = match self.tasting_notes {
            Some(raw) => from_str::<Vec<String>>(&raw).map_err(|err| {
                RepositoryError::unexpected(format!("failed to decode tasting notes: {err}"))
            })?,
            None => Vec::new(),
        };

        Ok(Roast {
            id: RoastId::from(self.id),
            roaster_id: RoasterId::from(self.roaster_id),
            name: self.name,
            slug: self.slug,
            origin: self.origin,
            region: self.region,
            producer: self.producer,
            process: self.process,
            tasting_notes,
            created_at: self.created_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RoastWithRoasterRecord {
    id: i64,
    roaster_id: i64,
    name: String,
    slug: String,
    origin: Option<String>,
    region: Option<String>,
    producer: Option<String>,
    process: Option<String>,
    tasting_notes: Option<String>,
    created_at: DateTime<Utc>,
    roaster_name: String,
    roaster_slug: String,
}

impl RoastWithRoasterRecord {
    fn into_with_roaster(self) -> Result<RoastWithRoaster, RepositoryError> {
        let tasting_notes = match self.tasting_notes {
            Some(raw) => from_str::<Vec<String>>(&raw).map_err(|err| {
                RepositoryError::unexpected(format!("failed to decode tasting notes: {err}"))
            })?,
            None => Vec::new(),
        };

        Ok(RoastWithRoaster {
            roast: Roast {
                id: RoastId::from(self.id),
                roaster_id: RoasterId::from(self.roaster_id),
                name: self.name,
                slug: self.slug,
                origin: self.origin,
                region: self.region,
                producer: self.producer,
                process: self.process,
                tasting_notes,
                created_at: self.created_at,
            },
            roaster_name: self.roaster_name,
            roaster_slug: self.roaster_slug,
        })
    }
}
