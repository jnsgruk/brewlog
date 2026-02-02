use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;

use super::macros::{define_delete_handler, define_enriched_get_handler};
use crate::application::auth::AuthenticatedUser;
use crate::application::errors::{ApiError, AppError, map_app_error};
use crate::application::routes::render_html;
use crate::application::routes::support::{
    FlexiblePayload, ListQuery, PayloadSource, is_datastar_request,
};
use crate::application::server::AppState;
use crate::domain::bags::BagFilter;
use crate::domain::brews::{BrewFilter, BrewSortKey, BrewWithDetails, NewBrew};
use crate::domain::gear::{GearCategory, GearFilter, GearSortKey};
use crate::domain::ids::{BagId, BrewId, GearId};
use crate::domain::listing::{ListRequest, SortDirection};
use crate::domain::timeline::{NewTimelineEvent, TimelineEventDetail};
use crate::presentation::web::templates::{BrewListTemplate, BrewsTemplate};
use crate::presentation::web::views::{
    BagOptionView, BrewView, GearOptionView, ListNavigator, Paginated,
};

const BREW_PAGE_PATH: &str = "/brews";
const BREW_FRAGMENT_PATH: &str = "/brews#brew-list";

struct BrewPageData {
    brews: Paginated<BrewView>,
    navigator: ListNavigator<BrewSortKey>,
}

#[tracing::instrument(skip(state))]
async fn load_brew_page(
    state: &AppState,
    request: ListRequest<BrewSortKey>,
) -> Result<BrewPageData, AppError> {
    let page = state
        .brew_repo
        .list(BrewFilter::all(), &request)
        .await
        .map_err(AppError::from)?;

    let (brews, navigator) = crate::application::routes::support::build_page_view(
        page,
        request,
        BrewView::from_domain,
        BREW_PAGE_PATH,
        BREW_FRAGMENT_PATH,
    );

    Ok(BrewPageData { brews, navigator })
}

#[tracing::instrument(skip(state, cookies, headers, query))]
pub(crate) async fn brews_page(
    State(state): State<AppState>,
    cookies: tower_cookies::Cookies,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Response, StatusCode> {
    let request = query.into_request::<BrewSortKey>();

    if is_datastar_request(&headers) {
        let is_authenticated = super::is_authenticated(&state, &cookies).await;
        return render_brew_list_fragment(state, request, is_authenticated)
            .await
            .map_err(map_app_error);
    }

    // Load open bags for the form dropdown
    let open_bags_request = ListRequest::show_all(
        crate::domain::bags::BagSortKey::RoastDate,
        SortDirection::Desc,
    );
    let open_bags = state
        .bag_repo
        .list(BagFilter::open(), &open_bags_request)
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;
    let bag_options: Vec<BagOptionView> = open_bags
        .items
        .into_iter()
        .map(BagOptionView::from)
        .collect();

    // Load grinders for dropdown
    let grinder_request = ListRequest::show_all(GearSortKey::Make, SortDirection::Asc);
    let grinders = state
        .gear_repo
        .list(
            GearFilter::for_category(GearCategory::Grinder),
            &grinder_request,
        )
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;
    let grinder_options: Vec<GearOptionView> = grinders
        .items
        .into_iter()
        .map(GearOptionView::from)
        .collect();

    // Load brewers for dropdown
    let brewer_request = ListRequest::show_all(GearSortKey::Make, SortDirection::Asc);
    let brewers = state
        .gear_repo
        .list(
            GearFilter::for_category(GearCategory::Brewer),
            &brewer_request,
        )
        .await
        .map_err(|err| map_app_error(AppError::from(err)))?;
    let brewer_options: Vec<GearOptionView> = brewers
        .items
        .into_iter()
        .map(GearOptionView::from)
        .collect();

    let BrewPageData { brews, navigator } = load_brew_page(&state, request)
        .await
        .map_err(map_app_error)?;

    let is_authenticated = super::is_authenticated(&state, &cookies).await;

    let template = BrewsTemplate {
        nav_active: "brews",
        is_authenticated,
        brews,
        bag_options,
        grinder_options,
        brewer_options,
        navigator,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewBrewSubmission {
    bag_id: BagId,
    coffee_weight: f64,
    grinder_id: GearId,
    grind_setting: f64,
    brewer_id: GearId,
    water_volume: i32,
    water_temp: f64,
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
            water_volume: self.water_volume,
            water_temp: self.water_temp,
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
    let request = query.into_request::<BrewSortKey>();
    let (submission, source) = payload.into_parts();
    let new_brew = submission.into_new_brew().map_err(ApiError::from)?;

    let brew = state
        .brew_repo
        .insert(new_brew)
        .await
        .map_err(AppError::from)?;

    // Fetch enriched brew details for timeline event
    let enriched = state
        .brew_repo
        .get_with_details(brew.id)
        .await
        .map_err(AppError::from)?;

    // Add timeline event
    let ratio = if enriched.brew.coffee_weight > 0.0 {
        format!(
            "1:{:.1}",
            f64::from(enriched.brew.water_volume) / enriched.brew.coffee_weight
        )
    } else {
        "N/A".to_string()
    };

    let event = NewTimelineEvent {
        entity_type: "brew".to_string(),
        entity_id: brew.id.into_inner(),
        action: "brewed".to_string(),
        occurred_at: chrono::Utc::now(),
        title: enriched.roast_name.clone(),
        details: vec![
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
            TimelineEventDetail {
                label: "Ratio".to_string(),
                value: ratio,
            },
        ],
        tasting_notes: vec![],
    };
    let _ = state.timeline_repo.insert(event).await;

    if is_datastar_request(&headers) {
        render_brew_list_fragment(state, request, true)
            .await
            .map_err(ApiError::from)
    } else if matches!(source, PayloadSource::Form) {
        let target = ListNavigator::new(BREW_PAGE_PATH, BREW_FRAGMENT_PATH, request).page_href(1);
        Ok(Redirect::to(&target).into_response())
    } else {
        Ok((StatusCode::CREATED, Json(enriched)).into_response())
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
        .list(filter, &request)
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
    is_authenticated: bool,
) -> Result<Response, AppError> {
    let BrewPageData { brews, navigator } = load_brew_page(&state, request).await?;

    let template = BrewListTemplate {
        is_authenticated,
        brews,
        navigator,
    };

    crate::application::routes::support::render_fragment(template, "#brew-list")
}
