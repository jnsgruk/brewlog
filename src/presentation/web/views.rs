use crate::domain::bags::BagWithRoast;
use crate::domain::brews::{Brew, BrewWithDetails};
use crate::domain::cafes::Cafe;
use crate::domain::cups::CupWithDetails;
use crate::domain::gear::Gear;
use crate::domain::listing::{DEFAULT_PAGE_SIZE, ListRequest, Page, PageSize, SortKey};
use crate::domain::roasters::Roaster;
use crate::domain::roasts::{Roast, RoastWithRoaster};
use crate::domain::timeline::{TimelineEvent, TimelineEventDetail};

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
    base_path: &'static str,
    fragment_path: &'static str,
    request: ListRequest<K>,
    search: Option<String>,
}

impl<K: SortKey> ListNavigator<K> {
    pub fn new(
        base_path: &'static str,
        fragment_path: &'static str,
        request: ListRequest<K>,
        search: Option<String>,
    ) -> Self {
        Self {
            base_path,
            fragment_path,
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
        self.build_href(self.base_path, self.request.with_page(page))
    }

    pub fn fragment_page_href(&self, page: u32) -> String {
        self.build_href(self.fragment_path, self.request.with_page(page))
    }

    pub fn rows_href(&self, value: &str) -> String {
        self.build_href(self.base_path, Self::request_for_rows(self.request, value))
    }

    pub fn fragment_rows_href(&self, value: &str) -> String {
        self.build_href(
            self.fragment_path,
            Self::request_for_rows(self.request, value),
        )
    }

    pub fn sort_href(&self, key: &str) -> String {
        self.build_href(self.base_path, Self::request_for_sort(self.request, key))
    }

    pub fn fragment_sort_href(&self, key: &str) -> String {
        self.build_href(
            self.fragment_path,
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
        self.base_path
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

    fn build_href(&self, path: &str, request: ListRequest<K>) -> String {
        if let Some((base, fragment)) = path.split_once('#') {
            format!("{}?{}#{}", base, self.build_query_string(request), fragment)
        } else {
            format!("{}?{}", path, self.build_query_string(request))
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

pub struct RoasterOptionView {
    pub id: String,
    pub name: String,
}

impl From<Roaster> for RoasterOptionView {
    fn from(roaster: Roaster) -> Self {
        Self {
            id: roaster.id.to_string(),
            name: roaster.name,
        }
    }
}

impl From<&Roaster> for RoasterOptionView {
    fn from(roaster: &Roaster) -> Self {
        Self {
            id: roaster.id.to_string(),
            name: roaster.name.clone(),
        }
    }
}

pub struct RoasterView {
    pub id: String,
    pub detail_path: String,
    pub name: String,
    pub country: String,
    pub city: String,
    pub has_homepage: bool,
    pub homepage_url: String,
    pub homepage_label: String,
    pub notes: String,
    pub created_at: String,
    pub created_at_sort_key: i64,
}

impl From<Roaster> for RoasterView {
    fn from(roaster: Roaster) -> Self {
        let Roaster {
            id,
            slug,
            name,
            country,
            city,
            homepage,
            notes,
            created_at,
        } = roaster;

        let homepage = homepage.unwrap_or_default();
        let has_homepage = !homepage.is_empty();
        let detail_path = format!("/roasters/{slug}");

        let created_at_sort_key = created_at.timestamp();
        let created_at_label = created_at.format("%Y-%m-%d").to_string();

        Self {
            detail_path,
            id: id.to_string(),
            name,
            country,
            city: city.unwrap_or_else(|| "—".to_string()),
            has_homepage,
            homepage_url: homepage.clone(),
            homepage_label: homepage,
            notes: notes.unwrap_or_else(|| "This roaster has no notes yet.".to_string()),
            created_at: created_at_label,
            created_at_sort_key,
        }
    }
}

pub struct RoastView {
    pub id: String,
    pub full_id: String,
    pub detail_path: String,
    pub name: String,
    pub roaster_label: String,
    pub origin: String,
    pub region: String,
    pub producer: String,
    pub process: String,
    pub created_at: String,
    pub created_at_sort_key: i64,
    pub tasting_notes: Vec<String>,
}

impl RoastView {
    pub fn from_domain(roast: Roast, roaster_name: &str, roaster_slug: &str) -> Self {
        Self::from_parts(roast, roaster_name, roaster_slug)
    }

    pub fn from_list_item(item: RoastWithRoaster) -> Self {
        let RoastWithRoaster {
            roast,
            roaster_name,
            roaster_slug,
        } = item;
        Self::from_parts(roast, &roaster_name, &roaster_slug)
    }

    fn from_parts(roast: Roast, roaster_name: &str, roaster_slug: &str) -> Self {
        let Roast {
            id: roast_id,
            roaster_id: _,
            name,
            slug,
            origin,
            region,
            producer,
            tasting_notes,
            process,
            created_at,
        } = roast;

        let full_id = roast_id.to_string();
        let id: String = full_id.chars().take(6).collect();
        let roaster_label = if roaster_name.trim().is_empty() {
            "Unknown roaster".to_string()
        } else {
            roaster_name.to_string()
        };
        let origin = origin.unwrap_or_else(|| "—".to_string());
        let region = region.unwrap_or_else(|| "—".to_string());
        let producer = producer.unwrap_or_else(|| "—".to_string());
        let process = process.unwrap_or_else(|| "—".to_string());
        let created_at_sort_key = created_at.timestamp();
        let tasting_notes = tasting_notes
            .into_iter()
            .flat_map(|note| {
                note.split([',', '\n'])
                    .map(|segment| segment.trim().to_string())
                    .filter(|segment| !segment.is_empty())
                    .collect::<Vec<_>>()
            })
            .collect();
        let created_at = created_at.format("%Y-%m-%d").to_string();
        let detail_path = format!("/roasters/{roaster_slug}/roasts/{slug}");

        Self {
            id,
            full_id,
            detail_path,
            name,
            roaster_label,
            origin,
            region,
            producer,
            process,
            created_at,
            created_at_sort_key,
            tasting_notes,
        }
    }
}

#[derive(Clone)]
pub struct TimelineEventDetailView {
    pub label: String,
    pub value: String,
    pub link: Option<String>,
}

/// Raw brew data for repeating a brew from the timeline.
#[derive(Clone)]
pub struct TimelineBrewDataView {
    pub bag_id: i64,
    pub grinder_id: i64,
    pub brewer_id: i64,
    pub filter_paper_id: Option<i64>,
    pub coffee_weight: f64,
    pub grind_setting: f64,
    pub water_volume: i32,
    pub water_temp: f64,
}

#[derive(Clone)]
pub struct TimelineEventView {
    pub id: String,
    pub kind_label: &'static str,
    pub date_label: String,
    pub time_label: Option<String>,
    pub iso_timestamp: String,
    pub title: String,
    pub link: String,
    pub external_link: Option<String>,
    pub details: Vec<TimelineEventDetailView>,
    pub tasting_notes: Option<Vec<String>>,
    pub brew_data: Option<TimelineBrewDataView>,
}

pub struct TimelineMonthView {
    pub anchor: String,
    pub heading: String,
    pub events: Vec<TimelineEventView>,
}

impl TimelineEventView {
    pub fn from_domain(event: TimelineEvent) -> Self {
        let TimelineEvent {
            id,
            entity_type,
            entity_id,
            action,
            occurred_at,
            title,
            details,
            tasting_notes,
            slug,
            roaster_slug,
            brew_data,
        } = event;

        let kind_label = match (entity_type.as_str(), action.as_str()) {
            ("roaster", "added") => "Roaster Added",
            ("roast", "added") => "Roast Added",
            ("bag", "added") => "Bag Added",
            ("bag", "finished") => "Bag Finished",
            ("gear", "added") => "Gear Added",
            ("brew", "brewed") => "Brewed",
            ("cafe", "added") => "Cafe Added",
            ("cup", "added") => "Cup",
            _ => "Event",
        };

        let link = match (entity_type.as_str(), slug, roaster_slug) {
            ("roaster", Some(slug), _) => format!("/roasters/{slug}"),
            // Roasts, bags, brews, and cups link to the roast page when we have slug info
            ("roast" | "bag" | "brew" | "cup", Some(slug), Some(roaster_slug)) => {
                format!("/roasters/{roaster_slug}/roasts/{slug}")
            }
            ("cafe", Some(slug), _) => format!("/cafes/{slug}"),
            ("cup", _, _) => "/cups".to_string(),
            ("gear", _, _) => "/gear".to_string(),
            ("brew", _, _) => "/brews".to_string(),
            ("roaster", None, _) => format!("/roasters/{entity_id}"),
            ("roast", None, _) => format!("/roasts/{entity_id}"),
            _ => String::from("#"),
        };

        let (mapped_details, external_link) = Self::map_details(details);

        let tasting_notes = if entity_type == "roast" {
            let notes = tasting_notes
                .into_iter()
                .flat_map(|note| {
                    note.split([',', '\n'])
                        .map(|segment| segment.trim().to_string())
                        .filter(|segment| !segment.is_empty())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Some(notes)
        } else {
            None
        };

        let brew_data_view = brew_data.map(|bd| TimelineBrewDataView {
            bag_id: bd.bag_id,
            grinder_id: bd.grinder_id,
            brewer_id: bd.brewer_id,
            filter_paper_id: bd.filter_paper_id,
            coffee_weight: bd.coffee_weight,
            grind_setting: bd.grind_setting,
            water_volume: bd.water_volume,
            water_temp: bd.water_temp,
        });

        Self {
            id: id.to_string(),
            kind_label,
            date_label: occurred_at.format("%B %d, %Y").to_string(),
            time_label: Some(occurred_at.format("%H:%M UTC").to_string()),
            iso_timestamp: occurred_at.to_rfc3339(),
            title,
            link,
            external_link,
            details: mapped_details,
            tasting_notes,
            brew_data: brew_data_view,
        }
    }

    fn map_details(
        details: Vec<TimelineEventDetail>,
    ) -> (Vec<TimelineEventDetailView>, Option<String>) {
        let mut mapped = Vec::new();
        let mut external_link = None;
        for detail in details {
            if detail.label.eq_ignore_ascii_case("homepage")
                || detail.label.eq_ignore_ascii_case("website")
            {
                let trimmed = detail.value.trim();
                if !trimmed.is_empty() && trimmed != "—" {
                    external_link = Some(trimmed.to_string());
                }
            } else if detail.label.eq_ignore_ascii_case("position") {
                let trimmed = detail.value.trim();
                if !trimmed.is_empty() {
                    let display = trimmed
                        .strip_prefix("https://www.google.com/maps?q=")
                        .unwrap_or(trimmed);
                    mapped.push(TimelineEventDetailView {
                        label: detail.label,
                        value: display.to_string(),
                        link: Some(trimmed.to_string()),
                    });
                }
            } else {
                mapped.push(TimelineEventDetailView {
                    label: detail.label,
                    value: detail.value,
                    link: None,
                });
            }
        }
        (mapped, external_link)
    }
}

#[derive(Debug, Clone)]
pub struct BagView {
    pub id: String,
    pub roast_id: String,
    pub roast_date: Option<String>,
    pub amount: String,
    pub remaining: String,
    pub closed: bool,
    pub finished_at: String,
    pub created_at: String,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
}

impl BagView {
    pub fn from_domain(bag: BagWithRoast) -> Self {
        Self {
            id: bag.bag.id.to_string(),
            roast_id: bag.bag.roast_id.to_string(),
            roast_date: bag.bag.roast_date.map(|d| d.to_string()),
            amount: format!("{:.1}", bag.bag.amount),
            remaining: format!("{:.1}", bag.bag.remaining),
            closed: bag.bag.closed,
            finished_at: bag
                .bag
                .finished_at
                .map_or_else(|| "—".to_string(), |d| d.to_string()),
            created_at: bag.bag.created_at.format("%Y-%m-%d").to_string(),
            roast_name: bag.roast_name,
            roaster_name: bag.roaster_name,
            roast_slug: bag.roast_slug,
            roaster_slug: bag.roaster_slug,
        }
    }
}

#[derive(Clone)]
pub struct GearView {
    pub id: String,
    pub category: String,
    pub category_label: String,
    pub make: String,
    pub model: String,
    pub full_name: String,
    pub created_at: String,
}

impl GearView {
    pub fn from_domain(gear: crate::domain::gear::Gear) -> Self {
        Self {
            id: gear.id.to_string(),
            category: gear.category.as_str().to_string(),
            category_label: gear.category.display_label().to_string(),
            make: gear.make.clone(),
            model: gear.model.clone(),
            full_name: format!("{} {}", gear.make, gear.model),
            created_at: gear.created_at.format("%Y-%m-%d").to_string(),
        }
    }
}

#[derive(Clone)]
pub struct BrewView {
    pub id: String,
    pub bag_id: i64,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub coffee_weight: String,
    pub grinder_id: i64,
    pub grinder_name: String,
    pub grind_setting: String,
    pub brewer_id: i64,
    pub brewer_name: String,
    pub filter_paper_id: Option<i64>,
    pub filter_paper_name: Option<String>,
    pub water_volume: String,
    pub water_temp: String,
    pub ratio: String,
    pub created_at: String,
    // Raw values for "brew again" feature
    pub coffee_weight_raw: f64,
    pub grind_setting_raw: f64,
    pub water_volume_raw: i32,
    pub water_temp_raw: f64,
}

impl BrewView {
    pub fn from_domain(brew: BrewWithDetails) -> Self {
        let ratio = if brew.brew.coffee_weight > 0.0 {
            format!(
                "1:{:.1}",
                f64::from(brew.brew.water_volume) / brew.brew.coffee_weight
            )
        } else {
            "\u{2014}".to_string()
        };

        Self {
            id: brew.brew.id.to_string(),
            bag_id: brew.brew.bag_id.into_inner(),
            roast_name: brew.roast_name,
            roaster_name: brew.roaster_name,
            roast_slug: brew.roast_slug,
            roaster_slug: brew.roaster_slug,
            coffee_weight: format!("{:.1}g", brew.brew.coffee_weight),
            grinder_id: brew.brew.grinder_id.into_inner(),
            grinder_name: brew.grinder_name,
            grind_setting: format!("{:.1}", brew.brew.grind_setting),
            brewer_id: brew.brew.brewer_id.into_inner(),
            brewer_name: brew.brewer_name,
            filter_paper_id: brew
                .brew
                .filter_paper_id
                .map(crate::domain::ids::GearId::into_inner),
            filter_paper_name: brew.filter_paper_name,
            water_volume: format!("{}ml", brew.brew.water_volume),
            water_temp: format!("{:.1}\u{00B0}C", brew.brew.water_temp),
            ratio,
            created_at: brew.brew.created_at.format("%Y-%m-%d %H:%M").to_string(),
            coffee_weight_raw: brew.brew.coffee_weight,
            grind_setting_raw: brew.brew.grind_setting,
            water_volume_raw: brew.brew.water_volume,
            water_temp_raw: brew.brew.water_temp,
        }
    }
}

#[derive(Clone)]
pub struct BagOptionView {
    pub id: String,
    pub label: String,
}

impl From<BagWithRoast> for BagOptionView {
    fn from(bag: BagWithRoast) -> Self {
        Self {
            id: bag.bag.id.to_string(),
            label: format!(
                "{} - {} ({:.0}g remaining)",
                bag.roaster_name, bag.roast_name, bag.bag.remaining
            ),
        }
    }
}

#[derive(Clone)]
pub struct GearOptionView {
    pub id: String,
    pub label: String,
}

impl From<Gear> for GearOptionView {
    fn from(gear: Gear) -> Self {
        Self {
            id: gear.id.to_string(),
            label: format!("{} {}", gear.make, gear.model),
        }
    }
}

pub struct BrewDefaultsView {
    pub bag_id: String,
    pub grinder_id: String,
    pub brewer_id: String,
    pub filter_paper_id: String,
    pub coffee_weight: f64,
    pub grind_setting: f64,
    pub water_volume: i32,
    pub water_temp: f64,
}

impl Default for BrewDefaultsView {
    fn default() -> Self {
        Self {
            bag_id: String::new(),
            grinder_id: String::new(),
            brewer_id: String::new(),
            filter_paper_id: String::new(),
            coffee_weight: 15.0,
            grind_setting: 6.0,
            water_volume: 250,
            water_temp: 91.0,
        }
    }
}

impl From<Brew> for BrewDefaultsView {
    fn from(brew: Brew) -> Self {
        Self {
            bag_id: brew.bag_id.to_string(),
            grinder_id: brew.grinder_id.to_string(),
            brewer_id: brew.brewer_id.to_string(),
            filter_paper_id: brew
                .filter_paper_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            coffee_weight: brew.coffee_weight,
            grind_setting: brew.grind_setting,
            water_volume: brew.water_volume,
            water_temp: brew.water_temp,
        }
    }
}

pub struct CafeView {
    pub id: String,
    pub detail_path: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub map_url: String,
    pub has_website: bool,
    pub website_url: String,
    pub website_label: String,
    pub notes: String,
    pub created_at: String,
    pub created_at_sort_key: i64,
}

impl From<Cafe> for CafeView {
    fn from(cafe: Cafe) -> Self {
        let Cafe {
            id,
            slug,
            name,
            city,
            country,
            latitude,
            longitude,
            website,
            notes,
            created_at,
            updated_at: _,
        } = cafe;

        let website = website.unwrap_or_default();
        let has_website = !website.is_empty();
        let detail_path = format!("/cafes/{slug}");
        let map_url = format!("https://www.google.com/maps?q={latitude},{longitude}");

        let created_at_sort_key = created_at.timestamp();
        let created_at_label = created_at.format("%Y-%m-%d").to_string();

        Self {
            detail_path,
            id: id.to_string(),
            name,
            city,
            country,
            latitude,
            longitude,
            map_url,
            has_website,
            website_url: website.clone(),
            website_label: website,
            notes: notes.unwrap_or_else(|| "This cafe has no notes yet.".to_string()),
            created_at: created_at_label,
            created_at_sort_key,
        }
    }
}

pub struct CafeOptionView {
    pub id: String,
    pub label: String,
}

impl From<Cafe> for CafeOptionView {
    fn from(cafe: Cafe) -> Self {
        Self {
            id: cafe.id.to_string(),
            label: format!("{} ({})", cafe.name, cafe.city),
        }
    }
}

pub struct RoastOptionView {
    pub id: String,
    pub label: String,
}

impl From<RoastWithRoaster> for RoastOptionView {
    fn from(roast: RoastWithRoaster) -> Self {
        Self {
            id: roast.roast.id.to_string(),
            label: format!("{} - {}", roast.roaster_name, roast.roast.name),
        }
    }
}

#[derive(Clone)]
pub struct CupView {
    pub id: String,
    pub roast_name: String,
    pub roaster_name: String,
    pub roast_slug: String,
    pub roaster_slug: String,
    pub cafe_name: String,
    pub cafe_slug: String,
    pub notes: String,
    pub has_notes: bool,
    pub rating: String,
    pub has_rating: bool,
    pub created_at: String,
}

impl CupView {
    pub fn from_domain(cup: CupWithDetails) -> Self {
        let notes = cup.cup.notes.clone().unwrap_or_default();
        let has_notes = !notes.is_empty();
        let rating = cup
            .cup
            .rating
            .map_or_else(|| "—".to_string(), |r| format!("{r}/5"));
        let has_rating = cup.cup.rating.is_some();

        Self {
            id: cup.cup.id.to_string(),
            roast_name: cup.roast_name,
            roaster_name: cup.roaster_name,
            roast_slug: cup.roast_slug,
            roaster_slug: cup.roaster_slug,
            cafe_name: cup.cafe_name,
            cafe_slug: cup.cafe_slug,
            notes,
            has_notes,
            rating,
            has_rating,
            created_at: cup.cup.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
