use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Deserializer};
use tracing::{info, warn};

use super::macros::{define_delete_handler, define_enriched_get_handler};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError};
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::bags::BagFilter;
use crate::domain::brews::{BrewFilter, BrewSortKey, BrewWithDetails, NewBrew, QuickNote};
use crate::domain::gear::{GearCategory, GearFilter, GearSortKey};
use crate::domain::ids::{BagId, BrewId, GearId};
use crate::domain::listing::{ListRequest, PageSize, SortDirection};
use crate::domain::timeline::{NewTimelineEvent, TimelineBrewData, TimelineEventDetail};
use crate::presentation::web::templates::BrewListTemplate;
use crate::presentation::web::views::{
    BagOptionView, BrewDefaultsView, BrewView, GearOptionView, ListNavigator, Paginated,
    QuickNoteView,
};

const BREW_PAGE_PATH: &str = "/data?type=brews";
const BREW_FRAGMENT_PATH: &str = "/data?type=brews#brew-list";

pub(crate) struct BrewPageData {
    pub(crate) brews: Paginated<BrewView>,
    pub(crate) navigator: ListNavigator<BrewSortKey>,
}

pub(crate) struct BrewFormData {
    pub(crate) bag_options: Vec<BagOptionView>,
    pub(crate) grinder_options: Vec<GearOptionView>,
    pub(crate) brewer_options: Vec<GearOptionView>,
    pub(crate) filter_paper_options: Vec<GearOptionView>,
    pub(crate) defaults: BrewDefaultsView,
    pub(crate) quick_note_options: Vec<QuickNoteView>,
}

pub(crate) async fn load_brew_form_data(state: &AppState) -> Result<BrewFormData, AppError> {
    let open_bags_request = ListRequest::show_all(
        crate::domain::bags::BagSortKey::RoastDate,
        SortDirection::Desc,
    );
    let open_bags = state
        .bag_repo
        .list(BagFilter::open(), &open_bags_request, None)
        .await
        .map_err(AppError::from)?;
    let bag_options: Vec<BagOptionView> = open_bags
        .items
        .into_iter()
        .map(BagOptionView::from)
        .collect();

    let gear_request = ListRequest::show_all(GearSortKey::Make, SortDirection::Asc);

    let grinder_options = load_gear_options(state, GearCategory::Grinder, &gear_request).await?;
    let brewer_options = load_gear_options(state, GearCategory::Brewer, &gear_request).await?;
    let filter_paper_options =
        load_gear_options(state, GearCategory::FilterPaper, &gear_request).await?;

    let last_brew_request = ListRequest::new(
        1,
        PageSize::Limited(1),
        BrewSortKey::CreatedAt,
        SortDirection::Desc,
    );
    let last_brew_page = state
        .brew_repo
        .list(BrewFilter::all(), &last_brew_request, None)
        .await
        .map_err(AppError::from)?;
    let defaults = last_brew_page
        .items
        .into_iter()
        .next()
        .map(|b| BrewDefaultsView::from(b.brew))
        .unwrap_or_default();

    let quick_note_options = QuickNote::all()
        .iter()
        .copied()
        .map(QuickNoteView::from)
        .collect();

    Ok(BrewFormData {
        bag_options,
        grinder_options,
        brewer_options,
        filter_paper_options,
        defaults,
        quick_note_options,
    })
}

pub(crate) async fn load_gear_options(
    state: &AppState,
    category: GearCategory,
    request: &ListRequest<GearSortKey>,
) -> Result<Vec<GearOptionView>, AppError> {
    let page = state
        .gear_repo
        .list(GearFilter::for_category(category), request, None)
        .await
        .map_err(AppError::from)?;
    Ok(page.items.into_iter().map(GearOptionView::from).collect())
}

#[tracing::instrument(skip(state))]
pub(crate) async fn load_brew_page(
    state: &AppState,
    request: ListRequest<BrewSortKey>,
    search: Option<&str>,
) -> Result<BrewPageData, AppError> {
    let page = state
        .brew_repo
        .list(BrewFilter::all(), &request, search)
        .await
        .map_err(AppError::from)?;

    let (brews, navigator) = crate::application::routes::support::build_page_view(
        page,
        request,
        BrewView::from_domain,
        BREW_PAGE_PATH,
        BREW_FRAGMENT_PATH,
        search.map(String::from),
    );

    Ok(BrewPageData { brews, navigator })
}

/// Deserializes an optional `GearId`, treating empty strings (from HTML forms) as None.
fn deserialize_optional_gear_id<'de, D>(deserializer: D) -> Result<Option<GearId>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) if s.is_empty() => Ok(None),
        Some(serde_json::Value::Number(n)) => n
            .as_i64()
            .map(|id| Some(GearId::new(id)))
            .ok_or_else(|| serde::de::Error::custom("invalid gear id")),
        Some(serde_json::Value::String(s)) => s
            .parse::<i64>()
            .map(|id| Some(GearId::new(id)))
            .map_err(serde::de::Error::custom),
        Some(_) => Err(serde::de::Error::custom("invalid gear id")),
    }
}

fn deserialize_quick_notes<'de, D>(deserializer: D) -> Result<Vec<QuickNote>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(serde_json::Value::String(s)) if s.is_empty() => Ok(Vec::new()),
        Some(serde_json::Value::String(s)) => Ok(s
            .split(',')
            .filter_map(|v| QuickNote::from_str_value(v.trim()))
            .collect()),
        Some(serde_json::Value::Array(arr)) => Ok(arr
            .iter()
            .filter_map(|v| v.as_str().and_then(QuickNote::from_str_value))
            .collect()),
        Some(_) => Err(serde::de::Error::custom("invalid quick_notes")),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewBrewSubmission {
    bag_id: BagId,
    coffee_weight: f64,
    grinder_id: GearId,
    grind_setting: f64,
    brewer_id: GearId,
    #[serde(default, deserialize_with = "deserialize_optional_gear_id")]
    filter_paper_id: Option<GearId>,
    water_volume: i32,
    water_temp: f64,
    #[serde(default, deserialize_with = "deserialize_quick_notes")]
    quick_notes: Vec<QuickNote>,
}

impl NewBrewSubmission {
    fn into_new_brew(self) -> Result<NewBrew, AppError> {
        if self.coffee_weight <= 0.0 {
            return Err(AppError::validation("coffee weight must be positive"));
        }
        if self.grind_setting < 0.0 {
            return Err(AppError::validation("grind setting must be non-negative"));
        }
        if self.water_volume <= 0 {
            return Err(AppError::validation("water volume must be positive"));
        }
        if self.water_temp <= 0.0 || self.water_temp > 100.0 {
            return Err(AppError::validation(
                "water temperature must be between 0 and 100",
            ));
        }

        Ok(NewBrew {
            bag_id: self.bag_id,
            coffee_weight: self.coffee_weight,
            grinder_id: self.grinder_id,
            grind_setting: self.grind_setting,
            brewer_id: self.brewer_id,
            filter_paper_id: self.filter_paper_id,
            water_volume: self.water_volume,
            water_temp: self.water_temp,
            quick_notes: self.quick_notes,
        })
    }
}

#[tracing::instrument(skip(state, _auth_user, headers, query))]
pub(crate) async fn create_brew(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
    payload: FlexiblePayload<NewBrewSubmission>,
) -> Result<Response, ApiError> {
    let (request, search) = query.into_request_and_search::<BrewSortKey>();
    let (submission, source) = payload.into_parts();
    let new_brew = submission.into_new_brew().map_err(ApiError::from)?;

    let brew = state
        .brew_repo
        .insert(new_brew)
        .await
        .map_err(AppError::from)?;

    info!(brew_id = %brew.id, "brew created");

    // Fetch enriched brew details for timeline event
    let enriched = state
        .brew_repo
        .get_with_details(brew.id)
        .await
        .map_err(AppError::from)?;

    // Add timeline event
    if let Err(err) = state
        .timeline_repo
        .insert(brew_timeline_event(&enriched))
        .await
    {
        warn!(error = %err, entity_type = "brew", "failed to record timeline event");
    }

    if is_datastar_request(&headers) {
        // Check if request came from timeline - return a script that redirects
        let from_timeline = headers
            .get("referer")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|r| r.contains("/timeline"));

        if from_timeline {
            use axum::http::header::HeaderValue;
            let mut response =
                axum::response::Html("<script>window.location.reload()</script>").into_response();
            response
                .headers_mut()
                .insert("datastar-selector", HeaderValue::from_static("body"));
            response
                .headers_mut()
                .insert("datastar-mode", HeaderValue::from_static("append"));
            Ok(response)
        } else {
            render_brew_list_fragment(state, request, search, true)
                .await
                .map_err(ApiError::from)
        }
    } else if matches!(source, PayloadSource::Form) {
        let target =
            ListNavigator::new(BREW_PAGE_PATH, BREW_FRAGMENT_PATH, request, search).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(enriched)).into_response())
    }
}

fn brew_timeline_event(enriched: &BrewWithDetails) -> NewTimelineEvent {
    let ratio = if enriched.brew.coffee_weight > 0.0 {
        format!(
            "1:{:.1}",
            f64::from(enriched.brew.water_volume) / enriched.brew.coffee_weight
        )
    } else {
        "N/A".to_string()
    };

    let mut details = vec![
        TimelineEventDetail {
            label: "Roaster".to_string(),
            value: enriched.roaster_name.clone(),
        },
        TimelineEventDetail {
            label: "Coffee".to_string(),
            value: format!("{:.1}g", enriched.brew.coffee_weight),
        },
        TimelineEventDetail {
            label: "Water".to_string(),
            value: format!(
                "{}ml @ {:.1}\u{00B0}C",
                enriched.brew.water_volume, enriched.brew.water_temp
            ),
        },
        TimelineEventDetail {
            label: "Grinder".to_string(),
            value: format!(
                "{} @ {:.1}",
                enriched.grinder_name, enriched.brew.grind_setting
            ),
        },
        TimelineEventDetail {
            label: "Brewer".to_string(),
            value: enriched.brewer_name.clone(),
        },
    ];

    if let Some(ref fp_name) = enriched.filter_paper_name {
        details.push(TimelineEventDetail {
            label: "Filter".to_string(),
            value: fp_name.clone(),
        });
    }

    details.push(TimelineEventDetail {
        label: "Ratio".to_string(),
        value: ratio,
    });

    if !enriched.brew.quick_notes.is_empty() {
        let labels: Vec<&str> = enriched
            .brew
            .quick_notes
            .iter()
            .map(|n| n.label())
            .collect();
        details.push(TimelineEventDetail {
            label: "Notes".to_string(),
            value: labels.join(", "),
        });
    }

    NewTimelineEvent {
        entity_type: "brew".to_string(),
        entity_id: enriched.brew.id.into_inner(),
        action: "brewed".to_string(),
        occurred_at: chrono::Utc::now(),
        title: enriched.roast_name.clone(),
        details,
        tasting_notes: vec![],
        slug: Some(enriched.roast_slug.clone()),
        roaster_slug: Some(enriched.roaster_slug.clone()),
        brew_data: Some(TimelineBrewData {
            bag_id: enriched.brew.bag_id.into_inner(),
            grinder_id: enriched.brew.grinder_id.into_inner(),
            brewer_id: enriched.brew.brewer_id.into_inner(),
            filter_paper_id: enriched
                .brew
                .filter_paper_id
                .map(crate::domain::ids::GearId::into_inner),
            coffee_weight: enriched.brew.coffee_weight,
            grind_setting: enriched.brew.grind_setting,
            water_volume: enriched.brew.water_volume,
            water_temp: enriched.brew.water_temp,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct BrewsQuery {
    pub bag_id: Option<BagId>,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn list_brews(
    State(state): State<AppState>,
    Query(params): Query<BrewsQuery>,
) -> Result<Json<Vec<BrewWithDetails>>, ApiError> {
    let filter = match params.bag_id {
        Some(bag_id) => BrewFilter::for_bag(bag_id),
        None => BrewFilter::all(),
    };
    let request = ListRequest::show_all(BrewSortKey::CreatedAt, SortDirection::Desc);
    let page = state
        .brew_repo
        .list(filter, &request, None)
        .await
        .map_err(AppError::from)?;
    Ok(Json(page.items))
}

define_enriched_get_handler!(
    get_brew,
    BrewId,
    BrewWithDetails,
    brew_repo,
    get_with_details
);

define_delete_handler!(
    delete_brew,
    BrewId,
    BrewSortKey,
    brew_repo,
    render_brew_list_fragment
);

async fn render_brew_list_fragment(
    state: AppState,
    request: ListRequest<BrewSortKey>,
    search: Option<String>,
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let BrewPageData { brews, navigator } =
        load_brew_page(&state, request, search.as_deref()).await?;

    let template = BrewListTemplate {
        is_authenticated,
        brews,
        navigator,
    };

    crate::application::routes::support::render_fragment(template, "#brew-list")
}
