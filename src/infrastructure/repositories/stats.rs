use async_trait::async_trait;
use sqlx::query_as;

use crate::domain::RepositoryError;
use crate::domain::repositories::StatsRepository;
use crate::infrastructure::database::DatabasePool;

#[derive(Clone)]
pub struct SqlStatsRepository {
    pool: DatabasePool,
}

impl SqlStatsRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CountryCount {
    country: String,
    count: i64,
}

impl CountryCount {
    fn into_tuple(self) -> (String, u64) {
        (self.country, self.count as u64)
    }
}

#[async_trait]
impl StatsRepository for SqlStatsRepository {
    async fn roaster_country_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
        let rows = query_as::<_, CountryCount>(
            r"SELECT country, COUNT(*) as count
               FROM roasters
               GROUP BY country
               ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(rows.into_iter().map(CountryCount::into_tuple).collect())
    }

    async fn roast_origin_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
        let rows = query_as::<_, CountryCount>(
            r"SELECT origin as country, COUNT(*) as count
               FROM roasts
               WHERE origin IS NOT NULL AND origin != ''
               GROUP BY origin
               ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(rows.into_iter().map(CountryCount::into_tuple).collect())
    }

    async fn cup_country_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
        let rows = query_as::<_, CountryCount>(
            r"SELECT ca.country as country, COUNT(*) as count
               FROM cups c
               JOIN cafes ca ON c.cafe_id = ca.id
               GROUP BY ca.country
               ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(rows.into_iter().map(CountryCount::into_tuple).collect())
    }

    async fn cafe_country_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
        let rows = query_as::<_, CountryCount>(
            r"SELECT country, COUNT(*) as count
               FROM cafes
               GROUP BY country
               ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(rows.into_iter().map(CountryCount::into_tuple).collect())
    }
}
