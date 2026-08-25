use std::collections::HashMap;
use std::sync::LazyLock;

use isocountry::CountryCode;

static COUNTRY_NAMES: LazyLock<HashMap<String, CountryCode>> = LazyLock::new(|| {
    CountryCode::iter()
        .map(|country| (country.name().to_lowercase(), *country))
        .collect()
});

fn country_alias(name: &str) -> Option<CountryCode> {
    use CountryCode::{
        BOL, CIV, COD, COG, CZE, GBR, IRN, KOR, LAO, MKD, PRK, RUS, SWZ, SYR, TUR, TWN, TZA, USA,
        VEN, VNM,
    };

    match name {
        // Common names that differ from the ISO English short name.
        "bolivia" => Some(BOL),
        "cote d'ivoire" | "ivory coast" => Some(CIV),
        "czech republic" => Some(CZE),
        "iran" => Some(IRN),
        "laos" => Some(LAO),
        "north korea" => Some(PRK),
        "north macedonia" => Some(MKD),
        "russia" => Some(RUS),
        "south korea" | "korea" => Some(KOR),
        "syria" => Some(SYR),
        "taiwan" => Some(TWN),
        "tanzania" => Some(TZA),
        "turkiye" | "türkiye" => Some(TUR),
        "united kingdom" | "uk" | "england" | "scotland" | "wales" | "northern ireland" => {
            Some(GBR)
        }
        "united states" => Some(USA),
        "venezuela" => Some(VEN),
        "vietnam" => Some(VNM),
        "eswatini" => Some(SWZ),

        // Abbreviations used by existing Brewlog data.
        "drc" | "democratic republic of the congo" | "congo" => Some(COD),
        "republic of the congo" => Some(COG),
        "uae" => Some(CountryCode::ARE),
        _ => None,
    }
}

/// Resolve human-readable country names and ISO-3166-1 alpha-2/alpha-3 codes.
pub fn resolve_country(value: &str) -> Option<CountryCode> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let normalised = value.to_lowercase();
    country_alias(&normalised)
        .or_else(|| CountryCode::for_alpha2_caseless(value).ok())
        .or_else(|| CountryCode::for_alpha3_caseless(value).ok())
        .or_else(|| COUNTRY_NAMES.get(&normalised).copied())
}

/// Maps a country name or code to its ISO-3166-1 alpha-2 code.
pub fn country_to_iso(name: &str) -> Option<&'static str> {
    resolve_country(name).map(|country| country.alpha2())
}

/// Converts a validated country code to a flag emoji using regional indicator symbols.
pub fn country_flag_emoji(country: CountryCode) -> String {
    country
        .alpha2()
        .bytes()
        .filter_map(|byte| char::from_u32(0x1F1E6 + u32::from(byte - b'A')))
        .collect()
}

/// Resolve a country name or code directly to its flag emoji.
pub fn country_to_flag_emoji(value: &str) -> String {
    resolve_country(value)
        .map(country_flag_emoji)
        .unwrap_or_default()
}

/// Return Brewlog's preferred display name for a country.
pub fn country_display_name(country: CountryCode) -> &'static str {
    use CountryCode::{BOL, GBR, IRN, KOR, PRK, RUS, SYR, TWN, TZA, USA, VEN};

    match country {
        GBR => "United Kingdom",
        USA => "United States",
        KOR => "South Korea",
        PRK => "North Korea",
        TWN => "Taiwan",
        RUS => "Russia",
        IRN => "Iran",
        SYR => "Syria",
        VEN => "Venezuela",
        BOL => "Bolivia",
        TZA => "Tanzania",
        _ => country.name(),
    }
}

/// Resolve an ISO alpha-2 code to Brewlog's preferred display name.
pub fn country_name_from_iso(code: &str) -> Option<&'static str> {
    CountryCode::for_alpha2_caseless(code.trim())
        .ok()
        .map(country_display_name)
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
        .filter_map(resolve_country)
        .map(country_flag_emoji)
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
    fn official_names_and_codes_resolve() {
        for country in CountryCode::iter() {
            let expected_name = if *country == CountryCode::COG {
                // Preserve Brewlog's established coffee-origin convention.
                CountryCode::COD
            } else {
                *country
            };
            assert_eq!(resolve_country(country.name()), Some(expected_name));
            assert_eq!(resolve_country(country.alpha2()), Some(*country));
            assert_eq!(resolve_country(country.alpha3()), Some(*country));
        }
    }

    #[test]
    fn country_to_iso_handles_aliases() {
        let aliases = [
            ("United Kingdom", "GB"),
            ("UK", "GB"),
            ("England", "GB"),
            ("DRC", "CD"),
            ("Congo", "CD"),
            ("Republic of the Congo", "CG"),
            ("South Korea", "KR"),
            ("UAE", "AE"),
        ];

        for (name, expected) in aliases {
            assert_eq!(country_to_iso(name), Some(expected), "alias: {name}");
        }
    }

    #[test]
    fn country_to_iso_returns_none_for_unknown() {
        assert_eq!(country_to_iso("Blend"), None);
        assert_eq!(country_to_iso("Multiple Origins"), None);
        assert_eq!(country_to_iso(""), None);
    }

    #[test]
    fn country_flag_emoji_produces_correct_flags() {
        assert_eq!(country_to_flag_emoji("GB"), "🇬🇧");
        assert_eq!(country_to_flag_emoji("United States"), "🇺🇸");
        assert_eq!(country_to_flag_emoji("Ethiopia"), "🇪🇹");
        assert_eq!(country_to_flag_emoji("Narnia"), "");
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
        assert_eq!(flags, country_to_flag_emoji("ET"));
    }

    #[test]
    fn origins_to_flags_multiple() {
        let flags = origins_to_flags(Some("Ethiopia, Colombia"));
        let expected = format!(
            "{} {}",
            country_to_flag_emoji("ET"),
            country_to_flag_emoji("CO")
        );
        assert_eq!(flags, expected);
    }

    #[test]
    fn origins_to_flags_skips_unknown() {
        let flags = origins_to_flags(Some("Ethiopia, Narnia, Colombia"));
        let expected = format!(
            "{} {}",
            country_to_flag_emoji("ET"),
            country_to_flag_emoji("CO")
        );
        assert_eq!(flags, expected);
    }

    #[test]
    fn origins_to_flags_none() {
        assert_eq!(origins_to_flags(None), "");
    }
}
