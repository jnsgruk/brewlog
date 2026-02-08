use async_trait::async_trait;
use sqlx::{Row, query_as, query_scalar};
use tracing::info;

use crate::domain::RepositoryError;
use crate::domain::repositories::StatsRepository;
use crate::domain::stats::{BrewingSummaryStats, CachedStats, ConsumptionStats, RoastSummaryStats};
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

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct NameCount {
    name: String,
    count: i64,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct NameWeight {
    name: String,
    total_grams: f64,
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

    async fn roast_summary(&self) -> Result<RoastSummaryStats, RepositoryError> {
        let unique_origins: i64 = query_scalar(
            r"SELECT COUNT(DISTINCT origin) FROM roasts
               WHERE origin IS NOT NULL AND origin != ''",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let top_origin = query_as::<_, NameCount>(
            r"SELECT origin as name, COUNT(*) as count FROM roasts
               WHERE origin IS NOT NULL AND origin != ''
               GROUP BY origin ORDER BY count DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .map(|r| r.name);

        let top_roaster = query_as::<_, NameCount>(
            r"SELECT ro.name as name, COUNT(*) as count
               FROM roasts r JOIN roasters ro ON r.roaster_id = ro.id
               GROUP BY ro.id ORDER BY count DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .map(|r| r.name);

        let all_origin_counts = self.roast_origin_counts().await?;
        let origin_counts: Vec<(String, u64)> = all_origin_counts.into_iter().take(5).collect();
        let max_origin_count = origin_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);

        let all_flavour_counts: Vec<(String, u64)> = query_as::<_, NameCount>(
            r"WITH RECURSIVE raw(val) AS (
                SELECT TRIM(j.value)
                FROM roasts, json_each(roasts.tasting_notes) j
                WHERE roasts.tasting_notes IS NOT NULL AND roasts.tasting_notes != '[]'
              ),
              split(note, rest) AS (
                SELECT TRIM(SUBSTR(val, 1, INSTR(val || ',', ',') - 1)),
                       TRIM(SUBSTR(val, INSTR(val || ',', ',') + 1))
                FROM raw
                UNION ALL
                SELECT TRIM(SUBSTR(rest, 1, INSTR(rest || ',', ',') - 1)),
                       TRIM(SUBSTR(rest, INSTR(rest || ',', ',') + 1))
                FROM split WHERE rest != ''
              )
              SELECT note as name, COUNT(*) as count
              FROM split WHERE note != ''
              GROUP BY LOWER(note) ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .into_iter()
        .map(|r| (r.name, r.count as u64))
        .collect();

        let flavour_counts: Vec<(String, u64)> = all_flavour_counts.into_iter().take(5).collect();
        let max_flavour_count = flavour_counts.iter().map(|(_, c)| *c).max().unwrap_or(0);

        Ok(RoastSummaryStats {
            unique_origins: unique_origins as u64,
            top_origin,
            top_roaster,
            origin_counts,
            max_origin_count,
            flavour_counts,
            max_flavour_count,
        })
    }

    async fn consumption_summary(&self) -> Result<ConsumptionStats, RepositoryError> {
        let last_30_days_grams: f64 = query_scalar(
            r"SELECT COALESCE(SUM(coffee_weight), 0.0) FROM brews
               WHERE created_at >= datetime('now', '-30 days')",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let all_time_grams: f64 =
            query_scalar(r"SELECT COALESCE(SUM(coffee_weight), 0.0) FROM brews")
                .fetch_one(&self.pool)
                .await
                .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let brews_last_30_days: i64 = query_scalar(
            r"SELECT COUNT(*) FROM brews
               WHERE created_at >= datetime('now', '-30 days')",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        let brews_all_time: i64 = query_scalar(r"SELECT COUNT(*) FROM brews")
            .fetch_one(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        Ok(ConsumptionStats {
            last_30_days_grams,
            all_time_grams,
            brews_last_30_days: brews_last_30_days as u64,
            brews_all_time: brews_all_time as u64,
        })
    }

    async fn brewing_summary(&self) -> Result<BrewingSummaryStats, RepositoryError> {
        let brewer_counts: Vec<(String, u64)> = query_as::<_, NameCount>(
            r"SELECT g.make || ' ' || g.model as name, COUNT(*) as count
               FROM brews b JOIN gear g ON b.brewer_id = g.id
               GROUP BY g.id ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .into_iter()
        .map(|r| (r.name, r.count as u64))
        .collect();

        let grinder_counts: Vec<(String, u64)> = query_as::<_, NameCount>(
            r"SELECT g.make || ' ' || g.model as name, COUNT(*) as count
               FROM brews b JOIN gear g ON b.grinder_id = g.id
               GROUP BY g.id ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .into_iter()
        .map(|r| (r.name, r.count as u64))
        .collect();

        let grinder_weight_counts: Vec<(String, f64)> = query_as::<_, NameWeight>(
            r"SELECT g.make || ' ' || g.model as name,
                     ROUND(COALESCE(SUM(b.coffee_weight), 0), 1) as total_grams
               FROM brews b JOIN gear g ON b.grinder_id = g.id
               GROUP BY g.id ORDER BY total_grams DESC
               LIMIT 5",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .into_iter()
        .map(|r| (r.name, r.total_grams))
        .collect();

        let max_grinder_weight = grinder_weight_counts
            .iter()
            .map(|(_, g)| *g)
            .fold(0.0_f64, f64::max);

        let brew_time_distribution: Vec<(String, u64)> = query_as::<_, NameCount>(
            r"SELECT
                 CASE
                   WHEN brew_time < 60  THEN '< 1:00'
                   WHEN brew_time < 90  THEN '1:00–1:30'
                   WHEN brew_time < 120 THEN '1:30–2:00'
                   WHEN brew_time < 150 THEN '2:00–2:30'
                   WHEN brew_time < 180 THEN '2:30–3:00'
                   ELSE '3:00+'
                 END as name,
                 COUNT(*) as count
               FROM brews
               WHERE brew_time IS NOT NULL
               GROUP BY name
               ORDER BY MIN(brew_time)",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?
        .into_iter()
        .map(|r| (r.name, r.count as u64))
        .collect();

        let max_brew_time_count = brew_time_distribution
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(0);

        Ok(BrewingSummaryStats {
            brewer_counts,
            grinder_counts,
            grinder_weight_counts,
            max_grinder_weight,
            brew_time_distribution,
            max_brew_time_count,
        })
    }

    async fn get_cached(&self) -> Result<Option<CachedStats>, RepositoryError> {
        let row = sqlx::query(r"SELECT data FROM stats_cache WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        match row {
            Some(row) => {
                let json: String = row
                    .try_get("data")
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
                let stats: CachedStats = serde_json::from_str(&json)
                    .map_err(|err| RepositoryError::unexpected(err.to_string()))?;
                Ok(Some(stats))
            }
            None => Ok(None),
        }
    }

    async fn store_cached(&self, stats: &CachedStats) -> Result<(), RepositoryError> {
        let json = serde_json::to_string(stats)
            .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        sqlx::query(
            r"INSERT OR REPLACE INTO stats_cache (id, data, computed_at)
              VALUES (1, ?, datetime('now'))",
        )
        .bind(&json)
        .execute(&self.pool)
        .await
        .map_err(|err| RepositoryError::unexpected(err.to_string()))?;

        info!("stats cache updated");
        Ok(())
    }
}
