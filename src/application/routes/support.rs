use askama::Template;
use axum::extract::{Form, FromRequest, Json as JsonPayload, Request};
use axum::http::{HeaderMap, HeaderValue, header::CONTENT_TYPE};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;
use tracing::warn;

use crate::application::errors::{ApiError, AppError};
use crate::application::state::AppState;
use crate::domain::listing::{
    DEFAULT_PAGE_SIZE, ListRequest, Page, PageSize, SortDirection, SortKey,
};
use crate::presentation::web::views::{
    CafeOptionView, ListNavigator, Paginated, RoastOptionView, RoasterOptionView,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadSource {
    Json,
    Form,
}

#[derive(Debug)]
pub struct FlexiblePayload<T> {
    inner: T,
    source: PayloadSource,
}

impl<T> FlexiblePayload<T> {
    pub fn into_parts(self) -> (T, PayloadSource) {
        (self.inner, self.source)
    }
}

/// Trait for update structs that can report whether any field was set.
pub(crate) trait HasChanges {
    fn has_changes(&self) -> bool;
}

/// Implement `HasChanges` for an update struct by checking `is_some()` on each field.
macro_rules! impl_has_changes {
    ($type:ty, $($field:ident),+) => {
        impl $crate::application::routes::support::HasChanges for $type {
            fn has_changes(&self) -> bool {
                $(self.$field.is_some())||+
            }
        }
    };
}
pub(crate) use impl_has_changes;

/// Return `400 Bad Request` if neither the update struct nor the image has any changes.
pub(crate) fn validate_update<T: HasChanges>(
    update: &T,
    image: Option<&String>,
) -> Result<(), ApiError> {
    if !update.has_changes() && image.is_none() {
        return Err(AppError::validation("no changes provided").into());
    }
    Ok(())
}

/// Three-way response for update handlers: Datastar redirect, form redirect, or JSON.
pub(crate) fn update_response(
    headers: &HeaderMap,
    source: PayloadSource,
    detail_url: &str,
    json_body: Response,
) -> Result<Response, ApiError> {
    if is_datastar_request(headers) {
        render_redirect_script(detail_url).map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        Ok(Redirect::to(detail_url).into_response())
    } else {
        Ok(json_body)
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
    #[serde(default)]
    q: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PageSizeParam {
    Number(u32),
    Text(String),
}

impl ListQuery {
    pub fn search_value(&self) -> String {
        self.q.clone().unwrap_or_default()
    }

    pub fn into_request_and_search<K: SortKey>(self) -> (ListRequest<K>, Option<String>) {
        self.into_request_and_search_with_default::<K>(DEFAULT_PAGE_SIZE)
    }

    pub fn into_request_and_search_with_default<K: SortKey>(
        self,
        default_page_size: u32,
    ) -> (ListRequest<K>, Option<String>) {
        let ListQuery {
            page,
            page_size,
            sort_key,
            sort_dir,
            q,
        } = self;

        let search = q.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

        let page = page.unwrap_or(1);
        let page_size = match page_size {
            Some(PageSizeParam::Number(value)) => PageSize::limited(value),
            Some(PageSizeParam::Text(text)) => page_size_from_text(&text),
            None => PageSize::limited(default_page_size.max(1)),
        };

        let sk = sort_key
            .as_deref()
            .and_then(K::from_query)
            .unwrap_or_else(K::default);

        let sd = sort_dir
            .as_deref()
            .and_then(parse_direction)
            .unwrap_or_else(|| sk.default_direction());

        (ListRequest::new(page, page_size, sk, sd), search)
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
    base_path: impl Into<String>,
    fragment_path: impl Into<String>,
    search: Option<String>,
) -> (Paginated<V>, ListNavigator<K>)
where
    K: SortKey,
{
    let normalized_request = normalize_request(request, &page);
    let view_page = Paginated::from_page(page, view_mapper);
    let navigator = ListNavigator::new(base_path, fragment_path, normalized_request, search);
    (view_page, navigator)
}

/// Return a Datastar response that redirects the browser to `url`.
///
/// Works by appending a `<script>` tag to `<body>` that sets `window.location.href`.
/// The URL is JSON-encoded to prevent XSS via crafted URLs.
pub fn render_redirect_script(url: &str) -> Result<Response, AppError> {
    let encoded_url = serde_json::to_string(url)
        .map_err(|err| AppError::unexpected(format!("failed to encode redirect URL: {err}")))?;
    let script = format!("<script>window.location.href={encoded_url}</script>");
    let mut response = Html(script).into_response();
    response
        .headers_mut()
        .insert("datastar-selector", HeaderValue::from_static("body"));
    response
        .headers_mut()
        .insert("datastar-mode", HeaderValue::from_static("append"));
    Ok(response)
}

pub fn render_fragment<T: Template>(
    template: T,
    selector: &'static str,
) -> Result<Response, AppError> {
    let html = crate::presentation::web::templates::render_template(template)
        .map_err(|err| AppError::unexpected(format!("failed to render fragment: {err}")))?;

    let mut response = Html(html).into_response();
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));
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
            let JsonPayload(payload) =
                JsonPayload::<T>::from_request(req, state)
                    .await
                    .map_err(|_| {
                        warn!("failed to parse JSON payload");
                        ApiError::from(AppError::validation("invalid JSON payload"))
                    })?;

            return Ok(Self {
                inner: payload,
                source: PayloadSource::Json,
            });
        }

        if content_type.is_empty() || content_type.starts_with("application/x-www-form-urlencoded")
        {
            let Form(payload) = Form::<T>::from_request(req, state).await.map_err(|_| {
                warn!("failed to parse form payload");
                ApiError::from(AppError::validation("invalid form payload"))
            })?;

            return Ok(Self {
                inner: payload,
                source: PayloadSource::Form,
            });
        }

        Err(AppError::validation("unsupported content type").into())
    }
}

pub(crate) async fn load_roaster_options(
    state: &AppState,
) -> Result<Vec<RoasterOptionView>, AppError> {
    use crate::domain::roasters::RoasterSortKey;
    let roasters = state
        .roaster_repo
        .list_all_sorted(RoasterSortKey::Name, SortDirection::Asc)
        .await
        .map_err(AppError::from)?;
    Ok(roasters.into_iter().map(RoasterOptionView::from).collect())
}

pub(crate) async fn load_roast_options(state: &AppState) -> Result<Vec<RoastOptionView>, AppError> {
    let roasts = state.roast_repo.list_all().await.map_err(AppError::from)?;
    Ok(roasts.into_iter().map(RoastOptionView::from).collect())
}

pub(crate) async fn load_cafe_options(state: &AppState) -> Result<Vec<CafeOptionView>, AppError> {
    use crate::domain::cafes::CafeSortKey;
    let cafes = state
        .cafe_repo
        .list_all_sorted(CafeSortKey::Name, SortDirection::Asc)
        .await
        .map_err(AppError::from)?;
    Ok(cafes.into_iter().map(CafeOptionView::from).collect())
}

/// Record AI usage in the background. Failures are logged but do not affect the response.
pub fn record_ai_usage(
    repo: std::sync::Arc<dyn crate::domain::repositories::AiUsageRepository>,
    user_id: crate::domain::ids::UserId,
    model: &str,
    endpoint: &str,
    usage: Option<crate::infrastructure::ai::Usage>,
) {
    let Some(usage) = usage else { return };
    let new_usage = crate::domain::ai_usage::NewAiUsage {
        user_id,
        model: model.to_string(),
        endpoint: endpoint.to_string(),
        prompt_tokens: usage.prompt_tokens,
        completion_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
        cost: usage.cost,
    };
    tokio::spawn(async move {
        if let Err(err) = repo.insert(new_usage).await {
            tracing::warn!(error = %err, "failed to record AI usage");
        }
    });
}

pub fn is_datastar_request(headers: &HeaderMap) -> bool {
    headers
        .get("datastar-request")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

pub fn set_datastar_patch_headers(headers: &mut HeaderMap, selector: &'static str) {
    let _ = headers.insert("datastar-selector", HeaderValue::from_static(selector));
    let _ = headers.insert("datastar-mode", HeaderValue::from_static("replace"));
}

/// Return a JSON response that Datastar interprets as a signal patch.
///
/// Signal names may use kebab-case (`_roaster-name`); they are automatically
/// converted to camelCase (`_roasterName`) to match Datastar's internal store.
pub fn render_signals_json(signals: &[(&str, serde_json::Value)]) -> Result<Response, AppError> {
    let mut map = serde_json::Map::new();
    for (name, value) in signals {
        map.insert(kebab_to_camel(name), value.clone());
    }
    let body = serde_json::Value::Object(map).to_string();
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(response)
}

fn kebab_to_camel(s: &str) -> String {
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
