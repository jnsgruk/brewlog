use std::collections::HashMap;
use std::sync::LazyLock;

/// A single country's count for geographic statistics.
#[derive(Debug, Clone)]
pub struct CountryStat {
    pub country_name: String,
    pub iso_code: String,
    pub flag_emoji: String,
    pub count: u64,
}

/// Aggregated geographic stats for one entity type.
#[derive(Debug, Clone)]
pub struct GeoStats {
    pub entries: Vec<CountryStat>,
    pub total_countries: usize,
    pub max_count: u64,
}

static COUNTRY_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // Coffee-producing countries
        ("ethiopia", "ET"),
        ("colombia", "CO"),
        ("kenya", "KE"),
        ("brazil", "BR"),
        ("guatemala", "GT"),
        ("costa rica", "CR"),
        ("rwanda", "RW"),
        ("honduras", "HN"),
        ("panama", "PA"),
        ("mexico", "MX"),
        ("el salvador", "SV"),
        ("bolivia", "BO"),
        ("peru", "PE"),
        ("indonesia", "ID"),
        ("india", "IN"),
        ("vietnam", "VN"),
        ("myanmar", "MM"),
        ("china", "CN"),
        ("papua new guinea", "PG"),
        ("tanzania", "TZ"),
        ("uganda", "UG"),
        ("burundi", "BI"),
        ("democratic republic of the congo", "CD"),
        ("drc", "CD"),
        ("congo", "CD"),
        ("republic of the congo", "CG"),
        ("yemen", "YE"),
        ("nicaragua", "NI"),
        ("ecuador", "EC"),
        ("dominican republic", "DO"),
        ("haiti", "HT"),
        ("jamaica", "JM"),
        ("thailand", "TH"),
        ("laos", "LA"),
        ("philippines", "PH"),
        ("taiwan", "TW"),
        // European roaster/cafe countries
        ("united kingdom", "GB"),
        ("uk", "GB"),
        ("england", "GB"),
        ("scotland", "GB"),
        ("wales", "GB"),
        ("northern ireland", "GB"),
        ("germany", "DE"),
        ("france", "FR"),
        ("spain", "ES"),
        ("italy", "IT"),
        ("netherlands", "NL"),
        ("belgium", "BE"),
        ("denmark", "DK"),
        ("sweden", "SE"),
        ("norway", "NO"),
        ("finland", "FI"),
        ("switzerland", "CH"),
        ("austria", "AT"),
        ("portugal", "PT"),
        ("ireland", "IE"),
        ("poland", "PL"),
        ("czech republic", "CZ"),
        ("czechia", "CZ"),
        ("greece", "GR"),
        ("slovenia", "SI"),
        ("croatia", "HR"),
        ("romania", "RO"),
        ("hungary", "HU"),
        // North America
        ("united states", "US"),
        ("united states of america", "US"),
        ("usa", "US"),
        ("us", "US"),
        ("canada", "CA"),
        // Asia-Pacific
        ("japan", "JP"),
        ("south korea", "KR"),
        ("korea", "KR"),
        ("australia", "AU"),
        ("new zealand", "NZ"),
        ("singapore", "SG"),
        // Other
        ("south africa", "ZA"),
        ("turkey", "TR"),
        ("israel", "IL"),
        ("united arab emirates", "AE"),
        ("uae", "AE"),
    ])
});

/// Maps a free-text country name to its ISO-3166-1 alpha-2 code.
pub fn country_to_iso(name: &str) -> Option<&'static str> {
    COUNTRY_MAP
        .get(name.trim().to_lowercase().as_str())
        .copied()
}

/// Converts an ISO-3166-1 alpha-2 code to a flag emoji using regional indicator symbols.
pub fn iso_to_flag_emoji(code: &str) -> String {
    code.chars()
        .filter_map(|c| {
            let upper = c.to_ascii_uppercase();
            if upper.is_ascii_uppercase() {
                char::from_u32(0x1F1E6 + (upper as u32 - 'A' as u32))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_to_iso_normalises_case() {
        assert_eq!(country_to_iso("Ethiopia"), Some("ET"));
        assert_eq!(country_to_iso("ETHIOPIA"), Some("ET"));
        assert_eq!(country_to_iso("  ethiopia  "), Some("ET"));
    }

    #[test]
    fn country_to_iso_handles_aliases() {
        assert_eq!(country_to_iso("United Kingdom"), Some("GB"));
        assert_eq!(country_to_iso("UK"), Some("GB"));
        assert_eq!(country_to_iso("England"), Some("GB"));
    }

    #[test]
    fn country_to_iso_returns_none_for_unknown() {
        assert_eq!(country_to_iso("Blend"), None);
        assert_eq!(country_to_iso("Multiple Origins"), None);
        assert_eq!(country_to_iso(""), None);
    }

    #[test]
    fn iso_to_flag_emoji_produces_correct_flags() {
        assert_eq!(iso_to_flag_emoji("GB"), "🇬🇧");
        assert_eq!(iso_to_flag_emoji("US"), "🇺🇸");
        assert_eq!(iso_to_flag_emoji("ET"), "🇪🇹");
    }
}
