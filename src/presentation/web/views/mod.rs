mod bags;
mod brews;
mod cafes;
mod cups;
mod gear;
mod roasters;
mod roasts;
pub mod tasting_notes;
mod timeline;

pub use bags::{BagDetailView, BagOptionView, BagView};
pub use brews::{BrewDefaultsView, BrewDetailView, BrewView, QuickNoteView};
pub use cafes::{CafeDetailView, CafeOptionView, CafeView, NearbyCafeView};
pub use cups::{CupDetailView, CupView};
pub use gear::{GearDetailView, GearOptionView, GearView};
pub use roasters::{RoasterDetailView, RoasterOptionView, RoasterView};
pub use roasts::{RoastDetailView, RoastOptionView, RoastView};
pub use tasting_notes::TastingNoteView;
pub use timeline::{
    TimelineBrewDataView, TimelineEventDetailView, TimelineEventView, TimelineMonthView,
};

#[derive(Default)]
pub struct StatsView {
    pub brews: u64,
    pub roasts: u64,
    pub roasters: u64,
    pub cups: u64,
    pub cafes: u64,
    pub bags: u64,
}

impl StatsView {
    pub fn is_empty(&self) -> bool {
        self.brews == 0
            && self.roasts == 0
            && self.roasters == 0
            && self.cups == 0
            && self.cafes == 0
            && self.bags == 0
    }
}

pub struct StatCard {
    pub icon: &'static str,
    pub value: String,
    pub label: &'static str,
}

use chrono::{DateTime, Utc};

use crate::domain::listing::{DEFAULT_PAGE_SIZE, ListRequest, Page, PageSize, SortKey};

fn relative_date(dt: DateTime<Utc>) -> String {
    crate::domain::formatting::format_relative_time(dt, Utc::now())
}

pub(crate) fn format_datetime(dt: DateTime<Utc>) -> (String, String) {
    (
        dt.format("%Y-%m-%d").to_string(),
        dt.format("%H:%M").to_string(),
    )
}

pub struct Paginated<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub showing_all: bool,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total: u64, showing_all: bool) -> Self {
        let page = page.max(1);
        let page_size = page_size.max(1);
        Self {
            items,
            page,
            page_size,
            total,
            showing_all,
        }
    }

    pub fn from_page<U, MapFn>(page: Page<U>, map_item: MapFn) -> Self
    where
        MapFn: FnMut(U) -> T,
    {
        let items = page.items.into_iter().map(map_item).collect();

        Self::new(
            items,
            page.page,
            page.page_size,
            page.total,
            page.showing_all,
        )
    }

    pub fn total_pages(&self) -> u32 {
        if self.total == 0 || self.showing_all {
            1
        } else {
            let page_size = u64::from(self.page_size);
            self.total.div_ceil(page_size) as u32
        }
    }

    pub fn has_previous(&self) -> bool {
        !self.showing_all && self.page > 1
    }

    pub fn has_next(&self) -> bool {
        !self.showing_all && self.page < self.total_pages()
    }

    pub fn previous_page(&self) -> Option<u32> {
        if self.has_previous() {
            Some(self.page - 1)
        } else {
            None
        }
    }

    pub fn next_page(&self) -> Option<u32> {
        if self.has_next() {
            Some(self.page + 1)
        } else {
            None
        }
    }

    pub fn start_index(&self) -> u64 {
        if self.total == 0 {
            0
        } else {
            u64::from(self.page - 1) * u64::from(self.page_size) + 1
        }
    }

    pub fn end_index(&self) -> u64 {
        if self.total == 0 {
            0
        } else {
            self.start_index() + self.items.len() as u64 - 1
        }
    }

    pub fn is_showing_all(&self) -> bool {
        self.showing_all
    }

    pub fn is_page_size(&self, value: u32) -> bool {
        !self.showing_all && self.page_size == value
    }

    pub fn page_size_query_value(&self) -> String {
        if self.showing_all {
            "all".to_string()
        } else {
            self.page_size.to_string()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListNavigator<K: SortKey> {
    base_path: String,
    fragment_path: String,
    request: ListRequest<K>,
    search: Option<String>,
}

impl<K: SortKey> ListNavigator<K> {
    pub fn new(
        base_path: impl Into<String>,
        fragment_path: impl Into<String>,
        request: ListRequest<K>,
        search: Option<String>,
    ) -> Self {
        Self {
            base_path: base_path.into(),
            fragment_path: fragment_path.into(),
            request,
            search,
        }
    }

    pub const fn request(&self) -> ListRequest<K> {
        self.request
    }

    pub fn sort_key(&self) -> &'static str {
        self.request.sort_key().query_value()
    }

    pub fn sort_direction(&self) -> &'static str {
        self.request.sort_direction().as_str()
    }

    pub fn page(&self) -> u32 {
        self.request.page()
    }

    pub fn page_size_value(&self) -> String {
        self.request.page_size().to_query_value()
    }

    pub fn is_showing_all(&self) -> bool {
        self.request.page_size().is_all()
    }

    pub fn page_href(&self, page: u32) -> String {
        self.build_href(&self.base_path, self.request.with_page(page))
    }

    pub fn fragment_page_href(&self, page: u32) -> String {
        self.build_href(&self.fragment_path, self.request.with_page(page))
    }

    pub fn rows_href(&self, value: &str) -> String {
        self.build_href(&self.base_path, Self::request_for_rows(self.request, value))
    }

    pub fn fragment_rows_href(&self, value: &str) -> String {
        self.build_href(
            &self.fragment_path,
            Self::request_for_rows(self.request, value),
        )
    }

    pub fn sort_href(&self, key: &str) -> String {
        self.build_href(&self.base_path, Self::request_for_sort(self.request, key))
    }

    pub fn fragment_sort_href(&self, key: &str) -> String {
        self.build_href(
            &self.fragment_path,
            Self::request_for_sort(self.request, key),
        )
    }

    pub fn is_sorted_by(&self, key: &str) -> bool {
        K::from_query(key).is_some_and(|candidate| candidate == self.request.sort_key())
    }

    pub fn next_sort_dir(&self, key: &str) -> &'static str {
        let sort_key = K::from_query(key).unwrap_or_else(K::default);
        let direction = if sort_key == self.request.sort_key() {
            self.request.sort_direction().opposite()
        } else {
            sort_key.default_direction()
        };
        direction.as_str()
    }

    pub fn query(&self) -> String {
        self.build_query_string(self.request)
    }

    pub fn query_for_page(&self, page: u32) -> String {
        self.build_query_string(self.request.with_page(page))
    }

    pub fn query_for_rows(&self, value: &str) -> String {
        self.build_query_string(Self::request_for_rows(self.request, value))
    }

    pub fn query_for_sort(&self, key: &str) -> String {
        self.build_query_string(Self::request_for_sort(self.request, key))
    }

    pub fn search_value(&self) -> &str {
        self.search.as_deref().unwrap_or("")
    }

    pub fn has_search(&self) -> bool {
        self.search.is_some()
    }

    /// Returns the base path (e.g., "/roasters") without query or fragment.
    pub fn path(&self) -> &str {
        &self.base_path
    }

    /// Returns query params for search actions (page reset to 1, preserves `sort/page_size`).
    /// Does NOT include the `q` parameter — the template appends it dynamically from JS.
    pub fn search_query_base(&self) -> String {
        format!(
            "page=1&page_size={}&sort={}&dir={}",
            self.request.page_size().to_query_value(),
            self.request.sort_key().query_value(),
            self.request.sort_direction().as_str()
        )
    }

    /// Returns the full URL prefix for search: `{path}?{query_base}&q=` or `{path}&{query_base}&q=`
    /// depending on whether the base path already contains query parameters.
    pub fn search_href_prefix(&self) -> String {
        let sep = if self.base_path.contains('?') {
            '&'
        } else {
            '?'
        };
        format!("{}{sep}{}&q=", self.base_path, self.search_query_base())
    }

    fn build_href(&self, path: &str, request: ListRequest<K>) -> String {
        let qs = self.build_query_string(request);
        if let Some((base, fragment)) = path.split_once('#') {
            let sep = if base.contains('?') { '&' } else { '?' };
            format!("{base}{sep}{qs}#{fragment}")
        } else {
            let sep = if path.contains('?') { '&' } else { '?' };
            format!("{path}{sep}{qs}")
        }
    }

    fn request_for_rows(request: ListRequest<K>, value: &str) -> ListRequest<K> {
        let page_size = page_size_from_text(value);
        request.with_page(1).with_page_size(page_size)
    }

    fn request_for_sort(request: ListRequest<K>, key: &str) -> ListRequest<K> {
        let sort_key = K::from_query(key).unwrap_or_else(K::default);
        request.with_page(1).with_sort(sort_key)
    }

    fn build_query_string(&self, request: ListRequest<K>) -> String {
        let mut qs = format!(
            "page={}&page_size={}&sort={}&dir={}",
            request.page(),
            request.page_size().to_query_value(),
            request.sort_key().query_value(),
            request.sort_direction().as_str()
        );
        if let Some(ref q) = self.search {
            qs.push_str("&q=");
            qs.push_str(&encode_uri_component(q));
        }
        qs
    }
}

fn encode_uri_component(s: &str) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            _ => {
                let _ = write!(result, "%{byte:02X}");
            }
        }
    }
    result
}

fn page_size_from_text(value: &str) -> PageSize {
    if value.eq_ignore_ascii_case("all") {
        PageSize::All
    } else if let Ok(parsed) = value.parse::<u32>() {
        PageSize::limited(parsed)
    } else {
        PageSize::limited(DEFAULT_PAGE_SIZE)
    }
}

/// A label + opacity pair for map legend entries on detail pages.
pub struct LegendEntry {
    pub label: &'static str,
    pub opacity: &'static str,
}

/// Shared coffee info fields extracted from a `Roast` for detail pages.
pub(crate) struct CoffeeInfo {
    pub origin: String,
    pub origin_flag: String,
    pub region: String,
    pub producer: String,
    pub process: String,
    pub tasting_notes: Vec<tasting_notes::TastingNoteView>,
}

/// Build coffee info fields from a roast, using em dash for empty/missing values.
pub(crate) fn build_coffee_info(roast: &crate::domain::roasts::Roast) -> CoffeeInfo {
    use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};

    let em_dash = "\u{2014}".to_string();
    let origin = roast.origin.clone().unwrap_or_default();
    let origin_flag = country_to_iso(&origin)
        .map(iso_to_flag_emoji)
        .unwrap_or_default();

    let notes = tasting_notes::parse_and_categorize(&roast.tasting_notes);

    CoffeeInfo {
        origin: if origin.is_empty() {
            em_dash.clone()
        } else {
            origin
        },
        origin_flag,
        region: roast
            .region
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or(em_dash.clone()),
        producer: roast
            .producer
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or(em_dash.clone()),
        process: roast
            .process
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or(em_dash),
        tasting_notes: notes,
    }
}

/// Shared roaster info fields extracted from a `Roaster` for detail pages.
pub(crate) struct RoasterInfo {
    pub country: String,
    pub country_flag: String,
    pub city: Option<String>,
    pub homepage: Option<String>,
}

/// Build roaster info fields from a roaster.
pub(crate) fn build_roaster_info(roaster: &crate::domain::roasters::Roaster) -> RoasterInfo {
    use crate::domain::countries::{country_to_iso, iso_to_flag_emoji};

    let country_flag = country_to_iso(&roaster.country)
        .map(iso_to_flag_emoji)
        .unwrap_or_default();

    RoasterInfo {
        country: roaster.country.clone(),
        country_flag,
        city: roaster.city.clone(),
        homepage: roaster.homepage.clone(),
    }
}

/// Build origin + roaster country map data with standard legend entries.
///
/// Origin gets weight 2, roaster country gets weight 1. Returns the
/// `(data-countries, data-max, legend_entries)` tuple used by detail pages.
pub(crate) fn build_origin_roaster_map(
    origin: Option<&str>,
    roaster_country: &str,
) -> (String, u32, Vec<LegendEntry>) {
    let mut entries: Vec<(&str, u32)> = Vec::new();
    if let Some(o) = origin.filter(|o| !o.is_empty()) {
        entries.push((o, 2));
    }
    entries.push((roaster_country, 1));
    let (map_countries, map_max) = build_map_data(&entries);
    (
        map_countries,
        map_max,
        vec![
            LegendEntry {
                label: "Origin",
                opacity: "",
            },
            LegendEntry {
                label: "Roaster",
                opacity: "opacity-50",
            },
        ],
    )
}

/// Build `data-countries` and `data-max` values for the world-map component.
///
/// Accepts `(country_name, weight)` pairs where higher weights render darker.
/// Resolves country names to ISO codes, deduplicates by keeping the highest weight
/// per ISO code, and returns the attribute string and max value.
pub(crate) fn build_map_data(entries: &[(&str, u32)]) -> (String, u32) {
    use std::collections::HashMap;

    use crate::domain::countries::country_to_iso;

    let mut iso_weights: HashMap<&str, u32> = HashMap::new();

    for &(country_name, weight) in entries {
        if country_name.is_empty() {
            continue;
        }
        if let Some(iso) = country_to_iso(country_name) {
            let entry = iso_weights.entry(iso).or_insert(0);
            *entry = (*entry).max(weight);
        }
    }

    let mut max = 0u32;
    let parts: Vec<String> = iso_weights
        .iter()
        .map(|(iso, &w)| {
            max = max.max(w);
            format!("{iso}:{w}")
        })
        .collect();

    (parts.join(","), max)
}

/// Build a JSON string for Datastar `data-signals` attribute initialization.
///
/// Signal names may use kebab-case (`_roaster-name`); they are automatically
/// converted to camelCase (`_roasterName`) to match Datastar's internal store.
/// The returned string is a JSON object suitable for use in `data-signals="{{ signals_json }}"`.
/// Askama HTML-escapes `"` to `&quot;`, which the browser decodes before Datastar parses it.
pub fn build_signals_json(signals: &[(&str, serde_json::Value)]) -> String {
    let mut map = serde_json::Map::new();
    for (name, value) in signals {
        map.insert(signals_kebab_to_camel(name), value.clone());
    }
    serde_json::Value::Object(map).to_string()
}

fn signals_kebab_to_camel(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut cap_next = false;
    for c in s.chars() {
        if c == '-' {
            cap_next = true;
        } else if cap_next {
            result.push(c.to_ascii_uppercase());
            cap_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;

    use crate::domain::ids::{RoastId, RoasterId};
    use crate::domain::listing::{DEFAULT_PAGE_SIZE, PageSize};
    use crate::domain::roasts::Roast;

    // ── StatsView::is_empty ─────────────────────────────────────────

    #[test]
    fn stats_view_default_is_empty() {
        assert!(StatsView::default().is_empty());
    }

    #[test]
    fn stats_view_any_nonzero_field_is_not_empty() {
        let with_brews = StatsView {
            brews: 1,
            ..Default::default()
        };
        assert!(!with_brews.is_empty());

        let with_roasts = StatsView {
            roasts: 1,
            ..Default::default()
        };
        assert!(!with_roasts.is_empty());

        let with_roasters = StatsView {
            roasters: 1,
            ..Default::default()
        };
        assert!(!with_roasters.is_empty());

        let with_cups = StatsView {
            cups: 1,
            ..Default::default()
        };
        assert!(!with_cups.is_empty());

        let with_cafes = StatsView {
            cafes: 1,
            ..Default::default()
        };
        assert!(!with_cafes.is_empty());

        let with_bags = StatsView {
            bags: 1,
            ..Default::default()
        };
        assert!(!with_bags.is_empty());
    }

    // ── format_datetime ─────────────────────────────────────────────

    #[test]
    fn format_datetime_known_value() {
        let dt = Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();
        let (date, time) = format_datetime(dt);
        assert_eq!(date, "2024-03-15");
        assert_eq!(time, "14:30");
    }

    // ── Paginated ───────────────────────────────────────────────────

    #[test]
    fn total_pages_rounds_up() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 25, false);
        assert_eq!(p.total_pages(), 3);
    }

    #[test]
    fn total_pages_zero_total_is_1() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 0, false);
        assert_eq!(p.total_pages(), 1);
    }

    #[test]
    fn total_pages_showing_all_is_1() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 100, true);
        assert_eq!(p.total_pages(), 1);
    }

    #[test]
    fn has_previous_false_on_page_1() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 30, false);
        assert!(!p.has_previous());
    }

    #[test]
    fn has_previous_true_on_page_2() {
        let p: Paginated<()> = Paginated::new(vec![], 2, 10, 30, false);
        assert!(p.has_previous());
    }

    #[test]
    fn has_next_true_when_not_last_page() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 30, false);
        assert!(p.has_next());
    }

    #[test]
    fn has_next_false_on_last_page() {
        let p: Paginated<()> = Paginated::new(vec![], 3, 10, 30, false);
        assert!(!p.has_next());
    }

    #[test]
    fn start_end_index_page_1() {
        let items: Vec<i32> = (1..=10).collect();
        let p = Paginated::new(items, 1, 10, 25, false);
        assert_eq!(p.start_index(), 1);
        assert_eq!(p.end_index(), 10);
    }

    #[test]
    fn start_end_index_page_2() {
        let items: Vec<i32> = (1..=10).collect();
        let p = Paginated::new(items, 2, 10, 25, false);
        assert_eq!(p.start_index(), 11);
        assert_eq!(p.end_index(), 20);
    }

    #[test]
    fn start_end_index_empty() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 0, false);
        assert_eq!(p.start_index(), 0);
        assert_eq!(p.end_index(), 0);
    }

    #[test]
    fn page_size_query_value_showing_all() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 10, 50, true);
        assert_eq!(p.page_size_query_value(), "all");
    }

    #[test]
    fn page_size_query_value_limited() {
        let p: Paginated<()> = Paginated::new(vec![], 1, 25, 50, false);
        assert_eq!(p.page_size_query_value(), "25");
    }

    // ── encode_uri_component ────────────────────────────────────────

    #[test]
    fn encode_ascii_alphanumeric_passes_through() {
        assert_eq!(encode_uri_component("abc123XYZ"), "abc123XYZ");
    }

    #[test]
    fn encode_spaces_to_percent_20() {
        assert_eq!(encode_uri_component("hello world"), "hello%20world");
    }

    #[test]
    fn encode_special_chars() {
        assert_eq!(encode_uri_component("&"), "%26");
        assert_eq!(encode_uri_component("="), "%3D");
        assert_eq!(encode_uri_component("?"), "%3F");
        assert_eq!(encode_uri_component("a&b=c?d"), "a%26b%3Dc%3Fd");
    }

    #[test]
    fn encode_preserves_unreserved_chars() {
        assert_eq!(encode_uri_component("-_.~"), "-_.~");
    }

    // ── page_size_from_text ─────────────────────────────────────────

    #[test]
    fn page_size_from_text_all() {
        assert_eq!(page_size_from_text("all"), PageSize::All);
        assert_eq!(page_size_from_text("ALL"), PageSize::All);
    }

    #[test]
    fn page_size_from_text_number() {
        assert_eq!(page_size_from_text("25"), PageSize::limited(25));
    }

    #[test]
    fn page_size_from_text_garbage_returns_default() {
        assert_eq!(
            page_size_from_text("garbage"),
            PageSize::limited(DEFAULT_PAGE_SIZE)
        );
    }

    // ── build_map_data ──────────────────────────────────────────────

    #[test]
    fn build_map_data_single_valid_country() {
        let entries = vec![("Ethiopia", 1u32)];
        let (data, max) = build_map_data(&entries);
        assert_eq!(data, "ET:1");
        assert_eq!(max, 1);
    }

    #[test]
    fn build_map_data_duplicate_keeps_highest_weight() {
        let entries = vec![("Ethiopia", 1), ("Ethiopia", 3)];
        let (data, max) = build_map_data(&entries);
        assert_eq!(data, "ET:3");
        assert_eq!(max, 3);
    }

    #[test]
    fn build_map_data_empty_country_skipped() {
        let entries = vec![("", 1)];
        let (data, max) = build_map_data(&entries);
        assert_eq!(data, "");
        assert_eq!(max, 0);
    }

    #[test]
    fn build_map_data_unknown_country_skipped() {
        let entries = vec![("Narnia", 1)];
        let (data, max) = build_map_data(&entries);
        assert_eq!(data, "");
        assert_eq!(max, 0);
    }

    // ── build_coffee_info ───────────────────────────────────────────

    fn make_roast(
        origin: Option<&str>,
        region: Option<&str>,
        producer: Option<&str>,
        process: Option<&str>,
        tasting_notes: Vec<&str>,
    ) -> Roast {
        Roast {
            id: RoastId::new(1),
            roaster_id: RoasterId::new(1),
            name: "Test Roast".to_string(),
            slug: "test-roast".to_string(),
            origin: origin.map(String::from),
            region: region.map(String::from),
            producer: producer.map(String::from),
            tasting_notes: tasting_notes.into_iter().map(String::from).collect(),
            process: process.map(String::from),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn build_coffee_info_all_none_shows_em_dashes() {
        let roast = make_roast(None, None, None, None, vec![]);
        let info = build_coffee_info(&roast);

        let em_dash = "\u{2014}";
        assert_eq!(info.origin, em_dash);
        assert_eq!(info.region, em_dash);
        assert_eq!(info.producer, em_dash);
        assert_eq!(info.process, em_dash);
        assert!(info.tasting_notes.is_empty());
    }

    #[test]
    fn build_coffee_info_all_populated_no_em_dashes() {
        let roast = make_roast(
            Some("Ethiopia"),
            Some("Yirgacheffe"),
            Some("Konga"),
            Some("Washed"),
            vec!["Blueberry", "Jasmine"],
        );
        let info = build_coffee_info(&roast);

        let em_dash = "\u{2014}";
        assert_ne!(info.origin, em_dash);
        assert_ne!(info.region, em_dash);
        assert_ne!(info.producer, em_dash);
        assert_ne!(info.process, em_dash);
        assert_eq!(info.origin, "Ethiopia");
        assert_eq!(info.region, "Yirgacheffe");
        assert_eq!(info.producer, "Konga");
        assert_eq!(info.process, "Washed");
        assert!(!info.origin_flag.is_empty());
        assert_eq!(info.tasting_notes.len(), 2);
    }

    #[test]
    fn build_coffee_info_comma_separated_tasting_notes_split() {
        let roast = make_roast(None, None, None, None, vec!["Blueberry, Jasmine, Caramel"]);
        let info = build_coffee_info(&roast);

        assert_eq!(info.tasting_notes.len(), 3);
        assert_eq!(info.tasting_notes[0].label, "Blueberry");
        assert_eq!(info.tasting_notes[1].label, "Jasmine");
        assert_eq!(info.tasting_notes[2].label, "Caramel");
    }
}
