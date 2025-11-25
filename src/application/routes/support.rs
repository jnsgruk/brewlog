use askama::Template;
use axum::async_trait;
use axum::extract::{Form, FromRequest, Json as JsonPayload, Request};
use axum::http::{HeaderMap, HeaderValue, header::CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;

use crate::application::errors::{ApiError, AppError};
use crate::domain::listing::{
    DEFAULT_PAGE_SIZE, ListRequest, Page, PageSize, SortDirection, SortKey,
};
use crate::presentation::views::{ListNavigator, Paginated};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadSource {
    Json,
    Form,
}

pub struct FlexiblePayload<T> {
    inner: T,
    source: PayloadSource,
}

impl<T> FlexiblePayload<T> {
    pub fn into_parts(self) -> (T, PayloadSource) {
        (self.inner, self.source)
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ListQuery {
    page: Option<u32>,
    #[serde(default)]
    page_size: Option<PageSizeParam>,
    #[serde(default, rename = "sort")]
    sort_key: Option<String>,
    #[serde(default, rename = "dir")]
    sort_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PageSizeParam {
    Number(u32),
    Text(String),
}

impl ListQuery {
    pub fn into_request<K: SortKey>(self) -> ListRequest<K> {
        self.into_request_with_default::<K>(DEFAULT_PAGE_SIZE)
    }

    pub fn into_request_with_default<K: SortKey>(self, default_page_size: u32) -> ListRequest<K> {
        let page = self.page.unwrap_or(1);
        let page_size = match self.page_size {
            Some(PageSizeParam::Number(value)) => PageSize::limited(value),
            Some(PageSizeParam::Text(text)) => page_size_from_text(&text),
            None => PageSize::limited(default_page_size.max(1)),
        };

        let sort_key = self
            .sort_key
            .as_deref()
            .and_then(K::from_query)
            .unwrap_or_else(K::default);

        let sort_direction = self
            .sort_dir
            .as_deref()
            .and_then(parse_direction)
            .unwrap_or_else(|| sort_key.default_direction());

        ListRequest::new(page, page_size, sort_key, sort_direction)
    }
}

pub fn normalize_request<K, T>(request: ListRequest<K>, page: &Page<T>) -> ListRequest<K>
where
    K: SortKey,
{
    let page_size = if page.showing_all {
        PageSize::All
    } else {
        PageSize::limited(page.page_size)
    };

    ListRequest::new(
        page.page,
        page_size,
        request.sort_key(),
        request.sort_direction(),
    )
}

pub fn build_page_view<K, T, V>(
    page: Page<T>,
    request: ListRequest<K>,
    view_mapper: impl FnMut(T) -> V,
    base_path: &'static str,
    fragment_path: &'static str,
) -> (Paginated<V>, ListNavigator<K>)
where
    K: SortKey,
{
    let normalized_request = normalize_request(request, &page);
    let view_page = Paginated::from_page(page, view_mapper);
    let navigator = ListNavigator::new(base_path, fragment_path, normalized_request);
    (view_page, navigator)
}

pub fn render_fragment<T: Template>(
    template: T,
    selector: &'static str,
) -> Result<Response, AppError> {
    let html = crate::presentation::templates::render_template(template)
        .map_err(|err| AppError::unexpected(format!("failed to render fragment: {err}")))?;

    let mut response = Html(html).into_response();
    set_datastar_patch_headers(response.headers_mut(), selector);
    Ok(response)
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

fn parse_direction(value: &str) -> Option<SortDirection> {
    match value.to_ascii_lowercase().as_str() {
        "asc" => Some(SortDirection::Asc),
        "desc" => Some(SortDirection::Desc),
        _ => None,
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for FlexiblePayload<T>
where
    S: Send + Sync,
    T: Send + 'static,
    JsonPayload<T>: FromRequest<S>,
    Form<T>: FromRequest<S>,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();

        if content_type.starts_with("application/json") {
            let JsonPayload(payload) = JsonPayload::<T>::from_request(req, state)
                .await
                .map_err(|_| ApiError::from(AppError::validation("invalid JSON payload")))?;

            return Ok(Self {
                inner: payload,
                source: PayloadSource::Json,
            });
        }

        if content_type.is_empty() || content_type.starts_with("application/x-www-form-urlencoded")
        {
            let Form(payload) = Form::<T>::from_request(req, state)
                .await
                .map_err(|_| ApiError::from(AppError::validation("invalid form payload")))?;

            return Ok(Self {
                inner: payload,
                source: PayloadSource::Form,
            });
        }

        Err(AppError::validation("unsupported content type").into())
    }
}

pub fn is_datastar_request(headers: &HeaderMap) -> bool {
    headers
        .get("datastar-request")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn set_datastar_patch_headers(headers: &mut HeaderMap, selector: &'static str) {
    let _ = headers.insert("datastar-selector", HeaderValue::from_static(selector));
    let _ = headers.insert("datastar-mode", HeaderValue::from_static("replace"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_datastar_request_detects_correctly() {
        let mut headers = HeaderMap::new();
        headers.insert("datastar-request", HeaderValue::from_static("true"));

        assert!(is_datastar_request(&headers));
    }

    #[test]
    fn is_datastar_request_detects_true_flag_case_insensitively() {
        let mut headers = HeaderMap::new();
        headers.insert("datastar-request", HeaderValue::from_static("TrUe"));

        assert!(is_datastar_request(&headers));
    }

    #[test]
    fn is_datastar_request_defaults_to_false() {
        let mut headers = HeaderMap::new();
        headers.insert("datastar-request", HeaderValue::from_static("nope"));

        assert!(!is_datastar_request(&headers));
        assert!(!is_datastar_request(&HeaderMap::new()));
    }

    #[test]
    fn set_datastar_patch_headers_sets_expected_values() {
        let mut headers = HeaderMap::new();

        set_datastar_patch_headers(&mut headers, "body > div");

        assert_eq!(
            headers.get("datastar-selector"),
            Some(&HeaderValue::from_static("body > div"))
        );
        assert_eq!(
            headers.get("datastar-mode"),
            Some(&HeaderValue::from_static("replace"))
        );
    }
}
