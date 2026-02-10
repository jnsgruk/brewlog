use async_trait::async_trait;
use sqlx::{query, query_as};

use crate::domain::RepositoryError;
use crate::domain::images::EntityImage;
use crate::domain::repositories::ImageRepository;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlImageRepository {
    pool: DatabasePool,
}

impl SqlImageRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    fn into_domain(record: ImageRecord) -> EntityImage {
        EntityImage {
            entity_type: record.entity_type,
            entity_id: record.entity_id,
            content_type: record.content_type,
            image_data: record.image_data,
            thumbnail_data: record.thumbnail_data,
        }
    }

    fn thumbnail_to_domain(record: ThumbnailRecord) -> EntityImage {
        EntityImage {
            entity_type: record.entity_type,
            entity_id: record.entity_id,
            content_type: record.content_type,
            image_data: Vec::new(),
            thumbnail_data: record.thumbnail_data,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ImageRecord {
    entity_type: String,
    entity_id: i64,
    content_type: String,
    image_data: Vec<u8>,
    thumbnail_data: Vec<u8>,
}

#[derive(sqlx::FromRow)]
struct ThumbnailRecord {
    entity_type: String,
    entity_id: i64,
    content_type: String,
    thumbnail_data: Vec<u8>,
}

#[async_trait]
impl ImageRepository for SqlImageRepository {
    async fn upsert(&self, image: EntityImage) -> Result<(), RepositoryError> {
        query(
            r"INSERT INTO entity_images (entity_type, entity_id, content_type, image_data, thumbnail_data)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT (entity_type, entity_id)
               DO UPDATE SET content_type = excluded.content_type,
                             image_data = excluded.image_data,
                             thumbnail_data = excluded.thumbnail_data,
                             created_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
        )
        .bind(&image.entity_type)
        .bind(image.entity_id)
        .bind(&image.content_type)
        .bind(&image.image_data)
        .bind(&image.thumbnail_data)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::unexpected(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, entity_type: &str, entity_id: i64) -> Result<EntityImage, RepositoryError> {
        let record = query_as::<_, ImageRecord>(
            r"SELECT entity_type, entity_id, content_type, image_data, thumbnail_data
               FROM entity_images
               WHERE entity_type = ? AND entity_id = ?",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::unexpected(e.to_string()))?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Self::into_domain(record))
    }

    async fn get_thumbnail(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<EntityImage, RepositoryError> {
        let record = query_as::<_, ThumbnailRecord>(
            r"SELECT entity_type, entity_id, content_type, thumbnail_data
               FROM entity_images
               WHERE entity_type = ? AND entity_id = ?",
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::unexpected(e.to_string()))?
        .ok_or(RepositoryError::NotFound)?;

        Ok(Self::thumbnail_to_domain(record))
    }

    async fn delete(&self, entity_type: &str, entity_id: i64) -> Result<(), RepositoryError> {
        query(r"DELETE FROM entity_images WHERE entity_type = ? AND entity_id = ?")
            .bind(entity_type)
            .bind(entity_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::unexpected(e.to_string()))?;

        Ok(())
    }

    async fn has_image(&self, entity_type: &str, entity_id: i64) -> Result<bool, RepositoryError> {
        let row: (i64,) =
            query_as(r"SELECT COUNT(*) FROM entity_images WHERE entity_type = ? AND entity_id = ?")
                .bind(entity_type)
                .bind(entity_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| RepositoryError::unexpected(e.to_string()))?;

        Ok(row.0 > 0)
    }
}
