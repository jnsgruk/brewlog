use serde::{Deserialize, Serialize};

use crate::domain::country_stats::GeoStats;

/// Summary statistics for roasts: origins, flavours, and roasters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoastSummaryStats {
    pub unique_origins: u64,
    pub top_origin: Option<String>,
    pub top_roaster: Option<String>,
    pub origin_counts: Vec<(String, u64)>,
    pub max_origin_count: u64,
    pub flavour_counts: Vec<(String, u64)>,
    pub max_flavour_count: u64,
}

/// Coffee consumption totals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumptionStats {
    pub last_30_days_grams: f64,
    pub all_time_grams: f64,
    pub brews_last_30_days: u64,
    pub brews_all_time: u64,
}

/// Brewing equipment statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewingSummaryStats {
    pub brewer_counts: Vec<(String, u64)>,
    pub grinder_counts: Vec<(String, u64)>,
    #[serde(alias = "grinder_kg_counts")]
    pub grinder_weight_counts: Vec<(String, f64)>,
    #[serde(alias = "max_grinder_kg")]
    pub max_grinder_weight: f64,
    pub brew_time_distribution: Vec<(String, u64)>,
    pub max_brew_time_count: u64,
}

/// Pre-computed snapshot of all statistics, stored as JSON in the cache table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedStats {
    pub roast_summary: RoastSummaryStats,
    pub consumption: ConsumptionStats,
    pub brewing_summary: BrewingSummaryStats,
    pub geo_roasters: GeoStats,
    pub geo_roasts: GeoStats,
    pub geo_cups: GeoStats,
    pub geo_cafes: GeoStats,
    pub computed_at: String,
}
