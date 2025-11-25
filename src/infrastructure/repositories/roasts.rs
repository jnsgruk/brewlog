use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Error as SqlxError, QueryBuilder, query, query_as, query_scalar};

use crate::domain::RepositoryError;
use crate::domain::ids::generate_id;
use crate::domain::listing::{ListRequest, Page, PageSize, SortDirection};
use crate::domain::repositories::RoastRepository;
use crate::domain::roasts::{Roast, RoastSortKey, RoastWithRoaster, UpdateRoast};
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

    fn to_domain(record: RoastRecord) -> Result<Roast, RepositoryError> {
        let RoastRecord {
            id,
            roaster_id,
            name,
            origin,
            region,
            producer,
            process,
            tasting_notes,
            created_at,
        } = record;

        let tasting_notes = match tasting_notes {
            Some(raw) if !raw.is_empty() => serde_json::from_str(&raw).map_err(|err| {
                RepositoryError::unexpected(format!("failed to decode tasting notes: {err}"))
            })?,
            _ => Vec::new(),
        };

        Ok(Roast {
            id,
            roaster_id,
            name,
            origin,
            region,
            producer,
            tasting_notes,
            process,
            created_at,
        })
    }

    fn to_with_roaster(
        record: RoastWithRoasterRecord,
    ) -> Result<RoastWithRoaster, RepositoryError> {
        let RoastWithRoasterRecord {
            id,
            roaster_id,
            name,
            origin,
            region,
            producer,
            process,
            tasting_notes,
            created_at,
            roaster_name,
        } = record;

        let roast = Self::to_domain(RoastRecord {
            id,
            roaster_id,
            name,
            origin,
            region,
            producer,
            process,
            tasting_notes,
            created_at,
        })?;

        Ok(RoastWithRoaster {
            roast,
            roaster_name,
        })
    }

    async fn get_record(&self, id: &str) -> Result<Roast, RepositoryError> {
        let record = query_as::<_, RoastRecord>(
            "SELECT id, roaster_id, name, origin, region, producer, process, tasting_notes, created_at FROM roasts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let Some(record) = record else {
            return Err(RepositoryError::NotFound);
        };

        Self::to_domain(record)
    }

    fn encode_notes(notes: &[String]) -> Result<Option<String>, RepositoryError> {
        if notes.is_empty() {
            Ok(None)
        } else {
            serde_json::to_string(notes).map(Some).map_err(|err| {
                RepositoryError::unexpected(format!("failed to encode tasting notes: {err}"))
            })
        }
    }
}

fn roast_order_clause(request: &ListRequest<RoastSortKey>) -> String {
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

#[async_trait]
impl RoastRepository for SqlRoastRepository {
    async fn insert(&self, roast: Roast) -> Result<Roast, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let notes = Self::encode_notes(&roast.tasting_notes)?;

        query(
            "INSERT INTO roasts (id, roaster_id, name, origin, region, producer, process, tasting_notes, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&roast.id)
        .bind(&roast.roaster_id)
        .bind(&roast.name)
        .bind(&roast.origin)
        .bind(&roast.region)
        .bind(&roast.producer)
        .bind(&roast.process)
        .bind(notes.as_deref())
        .bind(roast.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|err| map_insert_error(err, "unknown roaster reference"))?;

        let roaster_name: Option<String> =
            sqlx::query_scalar("SELECT name FROM roasters WHERE id = ?")
                .bind(&roast.roaster_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let roaster_label = roaster_name.unwrap_or_else(|| "Unknown roaster".to_string());

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

        let details_json = serde_json::to_string(&details).map_err(|err| {
            RepositoryError::unexpected(format!("failed to encode timeline event details: {err}"))
        })?;

        let tasting_notes_json = if roast.tasting_notes.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&roast.tasting_notes).map_err(|err| {
                RepositoryError::unexpected(format!(
                    "failed to encode timeline event tasting notes: {err}"
                ))
            })?)
        };

        sqlx::query("INSERT INTO timeline_events (id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(generate_id())
            .bind("roast")
            .bind(&roast.id)
            .bind(roast.created_at)
            .bind(&roast.name)
            .bind(details_json)
            .bind(tasting_notes_json)
            .execute(&mut *tx)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(roast)
    }

    async fn get(&self, id: String) -> Result<Roast, RepositoryError> {
        self.get_record(&id).await
    }

    async fn list(
        &self,
        request: &ListRequest<RoastSortKey>,
    ) -> Result<Page<RoastWithRoaster>, RepositoryError> {
        let order_clause = roast_order_clause(request);

        match request.page_size() {
            PageSize::All => {
                let query = format!(
                    "SELECT r.id, r.roaster_id, r.name, r.origin, r.region, r.producer, r.process, r.tasting_notes, r.created_at, ro.name AS roaster_name \
                     FROM roasts r \
                     JOIN roasters ro ON ro.id = r.roaster_id \
                     ORDER BY {}",
                    order_clause
                );

                let records = query_as::<_, RoastWithRoasterRecord>(&query)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                let items = records
                    .into_iter()
                    .map(Self::to_with_roaster)
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
                    "SELECT r.id, r.roaster_id, r.name, r.origin, r.region, r.producer, r.process, r.tasting_notes, r.created_at, ro.name AS roaster_name \
                     FROM roasts r \
                     JOIN roasters ro ON ro.id = r.roaster_id \
                     ORDER BY {} \
                     LIMIT ? OFFSET ?",
                    order_clause
                );

                let mut records = query_as::<_, RoastWithRoasterRecord>(&query)
                    .bind(page_size_i64)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                let total: i64 = query_scalar::<_, i64>("SELECT COUNT(*) FROM roasts")
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

                if page > 1 && records.is_empty() && total > 0 {
                    let last_page = ((total + page_size_i64 - 1) / page_size_i64) as u32;
                    page = last_page.max(1);
                    let offset = ((page - 1) as i64).saturating_mul(page_size_i64);
                    records = query_as::<_, RoastWithRoasterRecord>(&query)
                        .bind(page_size_i64)
                        .bind(offset)
                        .fetch_all(&self.pool)
                        .await
                        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
                }

                let items = records
                    .into_iter()
                    .map(Self::to_with_roaster)
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Page::new(items, page, page_size, total as u64, false))
            }
        }
    }

    async fn list_by_roaster(
        &self,
        roaster_id: String,
    ) -> Result<Vec<RoastWithRoaster>, RepositoryError> {
        let records = query_as::<_, RoastWithRoasterRecord>(
            "SELECT r.id, r.roaster_id, r.name, r.origin, r.region, r.producer, r.process, r.tasting_notes, r.created_at, ro.name AS roaster_name \
             FROM roasts r \
             JOIN roasters ro ON ro.id = r.roaster_id \
             WHERE r.roaster_id = ? \
             ORDER BY r.created_at DESC",
        )
        .bind(roaster_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        records.into_iter().map(Self::to_with_roaster).collect()
    }

    async fn update(&self, id: String, changes: UpdateRoast) -> Result<Roast, RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let UpdateRoast {
            roaster_id,
            name,
            origin,
            region,
            producer,
            tasting_notes,
            process,
        } = changes;

        let mut builder = QueryBuilder::new("UPDATE roasts SET ");
        let mut updated = false;

        if let Some(roaster_id) = roaster_id {
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("roaster_id = ");
            builder.push_bind(roaster_id);
        }
        if let Some(name) = name {
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("name = ");
            builder.push_bind(name);
        }
        if let Some(origin) = origin {
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("origin = ");
            builder.push_bind(origin);
        }
        if let Some(region) = region {
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("region = ");
            builder.push_bind(region);
        }
        if let Some(producer) = producer {
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("producer = ");
            builder.push_bind(producer);
        }
        if let Some(process) = process {
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("process = ");
            builder.push_bind(process);
        }
        if let Some(tasting_notes) = tasting_notes {
            let notes = Self::encode_notes(&tasting_notes)?;
            if updated {
                builder.push(", ");
            }
            updated = true;
            builder.push("tasting_notes = ");
            builder.push_bind(notes);
        }

        if updated {
            builder.push(" WHERE id = ");
            builder.push_bind(&id);

            let result = builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

            if result.rows_affected() == 0 {
                return Err(RepositoryError::NotFound);
            }
        }

        tx.commit()
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        self.get_record(&id).await
    }

    async fn delete(&self, id: String) -> Result<(), RepositoryError> {
        let result = query("DELETE FROM roasts WHERE id = ?")
            .bind(&id)
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
    id: String,
    roaster_id: String,
    name: String,
    origin: Option<String>,
    region: Option<String>,
    producer: Option<String>,
    process: Option<String>,
    tasting_notes: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct RoastWithRoasterRecord {
    id: String,
    roaster_id: String,
    name: String,
    origin: Option<String>,
    region: Option<String>,
    producer: Option<String>,
    process: Option<String>,
    tasting_notes: Option<String>,
    created_at: DateTime<Utc>,
    roaster_name: String,
}
