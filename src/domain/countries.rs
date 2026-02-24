use std::collections::HashMap;
use std::sync::LazyLock;

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

/// Split a comma-separated origin string into trimmed, non-empty country names.
///
/// Returns an empty `Vec` for `None`, empty, or whitespace-only input.
/// Single origins like `"Ethiopia"` yield `vec!["Ethiopia"]`.
/// Blends like `"Ethiopia, Colombia"` yield `vec!["Ethiopia", "Colombia"]`.
pub fn parse_origins(origin: Option<&str>) -> Vec<&str> {
    match origin {
        Some(s) if !s.trim().is_empty() => s
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Resolve a comma-separated origin string to a space-separated flag emoji string.
///
/// Unknown countries are silently skipped. Returns empty string if no countries resolve.
pub fn origins_to_flags(origin: Option<&str>) -> String {
    let flags: Vec<String> = parse_origins(origin)
        .into_iter()
        .filter_map(country_to_iso)
        .map(iso_to_flag_emoji)
        .collect();
    flags.join(" ")
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

    #[test]
    fn parse_origins_single() {
        assert_eq!(parse_origins(Some("Ethiopia")), vec!["Ethiopia"]);
    }

    #[test]
    fn parse_origins_multiple() {
        assert_eq!(
            parse_origins(Some("Ethiopia, Colombia")),
            vec!["Ethiopia", "Colombia"]
        );
    }

    #[test]
    fn parse_origins_trims_whitespace() {
        assert_eq!(
            parse_origins(Some("  Ethiopia , Colombia  , Kenya ")),
            vec!["Ethiopia", "Colombia", "Kenya"]
        );
    }

    #[test]
    fn parse_origins_empty_and_none() {
        assert!(parse_origins(None).is_empty());
        assert!(parse_origins(Some("")).is_empty());
        assert!(parse_origins(Some("  ")).is_empty());
    }

    #[test]
    fn parse_origins_trailing_comma() {
        assert_eq!(parse_origins(Some("Ethiopia,")), vec!["Ethiopia"]);
    }

    #[test]
    fn origins_to_flags_single() {
        let flags = origins_to_flags(Some("Ethiopia"));
        assert_eq!(flags, iso_to_flag_emoji("ET"));
    }

    #[test]
    fn origins_to_flags_multiple() {
        let flags = origins_to_flags(Some("Ethiopia, Colombia"));
        let expected = format!("{} {}", iso_to_flag_emoji("ET"), iso_to_flag_emoji("CO"));
        assert_eq!(flags, expected);
    }

    #[test]
    fn origins_to_flags_skips_unknown() {
        let flags = origins_to_flags(Some("Ethiopia, Narnia, Colombia"));
        let expected = format!("{} {}", iso_to_flag_emoji("ET"), iso_to_flag_emoji("CO"));
        assert_eq!(flags, expected);
    }

    #[test]
    fn origins_to_flags_none() {
        assert_eq!(origins_to_flags(None), "");
    }
}
