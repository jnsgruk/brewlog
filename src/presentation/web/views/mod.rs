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

    let notes = roast
        .tasting_notes
        .iter()
        .flat_map(|note| {
            note.split([',', '\n'])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .map(|n| tasting_notes::categorize(&n))
        .collect();

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
