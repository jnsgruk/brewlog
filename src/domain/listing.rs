use std::cmp;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub const fn as_str(self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            SortDirection::Asc => SortDirection::Desc,
            SortDirection::Desc => SortDirection::Asc,
        }
    }
}

pub trait SortKey: Copy + Eq {
    fn default() -> Self;
    fn from_query(value: &str) -> Option<Self>;
    fn query_value(self) -> &'static str;
    fn default_direction(self) -> SortDirection;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PageSize {
    Limited(u32),
    All,
}

impl PageSize {
    pub fn limited(size: u32) -> Self {
        if size == 0 {
            PageSize::All
        } else {
            PageSize::Limited(size)
        }
    }

    pub const fn is_all(self) -> bool {
        matches!(self, PageSize::All)
    }

    pub const fn as_option(self) -> Option<u32> {
        match self {
            PageSize::Limited(value) => Some(value),
            PageSize::All => None,
        }
    }

    pub fn to_query_value(self) -> String {
        match self {
            PageSize::All => "all".to_string(),
            PageSize::Limited(value) => value.to_string(),
        }
    }
}

pub const DEFAULT_PAGE_SIZE: u32 = 10;
pub const MAX_PAGE_SIZE: u32 = 50;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ListRequest<K: SortKey> {
    pub page: u32,
    pub page_size: PageSize,
    pub sort_key: K,
    pub sort_direction: SortDirection,
}

impl<K: SortKey> ListRequest<K> {
    pub fn new(page: u32, page_size: PageSize, sort_key: K, sort_direction: SortDirection) -> Self {
        let page = page.max(1);
        let page_size = match page_size {
            PageSize::Limited(size) => {
                if size == 0 {
                    PageSize::All
                } else {
                    let clamped = cmp::min(size, MAX_PAGE_SIZE).max(1);
                    PageSize::Limited(clamped)
                }
            }
            PageSize::All => PageSize::All,
        };

        Self {
            page,
            page_size,
            sort_key,
            sort_direction,
        }
    }

    pub fn default_query() -> Self {
        let key = K::default();
        Self::new(
            1,
            PageSize::Limited(DEFAULT_PAGE_SIZE),
            key,
            key.default_direction(),
        )
    }

    pub fn show_all(sort_key: K, sort_direction: SortDirection) -> Self {
        Self::new(1, PageSize::All, sort_key, sort_direction)
    }

    pub const fn page(&self) -> u32 {
        self.page
    }

    pub const fn page_size(&self) -> PageSize {
        self.page_size
    }

    pub const fn sort_key(&self) -> K {
        self.sort_key
    }

    pub const fn sort_direction(&self) -> SortDirection {
        self.sort_direction
    }

    pub fn with_page(self, page: u32) -> Self {
        Self {
            page: page.max(1),
            ..self
        }
    }

    pub fn with_page_size(self, page_size: PageSize) -> Self {
        Self::new(self.page, page_size, self.sort_key, self.sort_direction)
    }

    pub fn with_sort(self, key: K) -> Self {
        let direction = if key == self.sort_key {
            self.sort_direction.opposite()
        } else {
            key.default_direction()
        };
        Self::new(self.page, self.page_size, key, direction)
    }

    pub fn with_sort_and_direction(self, key: K, direction: SortDirection) -> Self {
        Self::new(self.page, self.page_size, key, direction)
    }

    pub fn ensure_page_within(self, total: u64) -> Self {
        if matches!(self.page_size, PageSize::All) {
            return Self::new(1, PageSize::All, self.sort_key, self.sort_direction);
        }

        let Some(limit) = self.page_size.as_option() else {
            return self;
        };

        if total == 0 {
            return Self::new(1, self.page_size, self.sort_key, self.sort_direction);
        }

        let last_page = (total.div_ceil(u64::from(limit))) as u32;
        let adjusted_page = self.page.min(last_page.max(1));
        Self::new(
            adjusted_page,
            self.page_size,
            self.sort_key,
            self.sort_direction,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
    pub showing_all: bool,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total: u64, showing_all: bool) -> Self {
        Self {
            items,
            page: page.max(1),
            page_size: page_size.max(1),
            total,
            showing_all,
        }
    }

    pub fn total_pages(&self) -> u32 {
        if self.total == 0 || self.showing_all {
            1
        } else {
            let size = u64::from(self.page_size);
            (self.total.div_ceil(size)) as u32
        }
    }

    pub fn has_previous(&self) -> bool {
        !self.showing_all && self.page > 1
    }

    pub fn has_next(&self) -> bool {
        !self.showing_all && self.page < self.total_pages()
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
}
