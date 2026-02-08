use serde::{Deserialize, Serialize};

use super::countries::{country_to_iso, iso_to_flag_emoji};

/// A single country's count for geographic statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryStat {
    pub country_name: String,
    pub iso_code: String,
    pub flag_emoji: String,
    pub count: u64,
}

/// Aggregated geographic stats for one entity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoStats {
    pub entries: Vec<CountryStat>,
    pub total_countries: usize,
    pub max_count: u64,
}

impl GeoStats {
    /// Build from raw (`country_name`, count) pairs, resolving ISO codes and flag emoji.
    pub fn from_counts(raw: Vec<(String, u64)>) -> Self {
        let entries: Vec<CountryStat> = raw
            .into_iter()
            .map(|(name, count)| {
                let iso = country_to_iso(&name).unwrap_or("").to_string();
                let flag = if iso.is_empty() {
                    String::new()
                } else {
                    iso_to_flag_emoji(&iso)
                };
                CountryStat {
                    country_name: name,
                    iso_code: iso,
                    flag_emoji: flag,
                    count,
                }
            })
            .collect();

        let total_countries = entries.len();
        let max_count = entries.iter().map(|e| e.count).max().unwrap_or(0);

        Self {
            entries,
            total_countries,
            max_count,
        }
    }
}
