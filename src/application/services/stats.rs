use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tracing::{error, info};

use crate::domain::country_stats::GeoStats;
use crate::domain::repositories::StatsRepository;
use crate::domain::stats::CachedStats;

/// Sends invalidation signals to the background stats recomputer.
/// Non-blocking and fire-and-forget — safe to call from any handler.
#[derive(Clone)]
pub struct StatsInvalidator {
    tx: mpsc::Sender<()>,
}

impl StatsInvalidator {
    pub fn new(tx: mpsc::Sender<()>) -> Self {
        Self { tx }
    }

    /// Signal that stats need recomputation.
    pub fn invalidate(&self) {
        let _ = self.tx.try_send(());
    }
}

/// Listens for invalidation signals, debounces, and recomputes all stats.
/// Runs as a long-lived background task — spawn with `tokio::spawn`.
pub async fn stats_recomputation_task(
    mut rx: mpsc::Receiver<()>,
    stats_repo: Arc<dyn StatsRepository>,
    debounce: Duration,
) {
    loop {
        if rx.recv().await.is_none() {
            break;
        }

        // Debounce: wait then drain any accumulated signals
        tokio::time::sleep(debounce).await;
        while rx.try_recv().is_ok() {}

        match compute_all_stats(&*stats_repo).await {
            Ok(cached) => {
                if let Err(err) = stats_repo.store_cached(&cached).await {
                    error!(error = %err, "failed to store stats cache");
                }
            }
            Err(err) => error!(error = %err, "stats recomputation failed"),
        }
    }
}

/// Runs all stats queries and assembles a complete `CachedStats` snapshot.
/// Logs the total computation time on success.
pub async fn compute_all_stats(
    repo: &dyn StatsRepository,
) -> Result<CachedStats, crate::domain::RepositoryError> {
    let start = Instant::now();

    let (
        roast_summary,
        consumption,
        brewing_summary,
        roaster_counts,
        roast_counts,
        cup_counts,
        cafe_counts,
        entity_counts,
    ) = tokio::join!(
        repo.roast_summary(),
        repo.consumption_summary(),
        repo.brewing_summary(),
        repo.roaster_country_counts(),
        repo.roast_origin_counts(),
        repo.cup_country_counts(),
        repo.cafe_country_counts(),
        repo.entity_counts(),
    );

    let cached = CachedStats {
        roast_summary: roast_summary?,
        consumption: consumption?,
        brewing_summary: brewing_summary?,
        geo_roasters: GeoStats::from_counts(roaster_counts?),
        geo_roasts: GeoStats::from_counts(roast_counts?),
        geo_cups: GeoStats::from_counts(cup_counts?),
        geo_cafes: GeoStats::from_counts(cafe_counts?),
        computed_at: chrono::Utc::now().to_rfc3339(),
        entity_counts: entity_counts?,
    };

    info!(duration_ms = start.elapsed().as_millis(), "stats computed");
    Ok(cached)
}
