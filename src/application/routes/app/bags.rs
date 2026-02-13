use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::auth::AuthenticatedUser;
use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::routes::support::load_roast_options;
use crate::application::state::AppState;
use crate::domain::entity_type::EntityType;
use crate::domain::ids::BagId;
use crate::presentation::web::templates::{BagDetailTemplate, BagEditTemplate};
use crate::presentation::web::views::BagDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn bag_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<BagId>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let bag = state
        .bag_repo
        .get_with_roast(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let (roast, image_url) = tokio::try_join!(
        async {
            state
                .roast_repo
                .get(bag.bag.roast_id)
                .await
                .map_err(|e| map_app_error(e.into()))
        },
        async {
            Ok::<_, StatusCode>(
                resolve_image_url(&state, EntityType::Roast, i64::from(bag.bag.roast_id)).await,
            )
        },
    )?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let view = BagDetailView::from_parts(bag, &roast, &roaster);

    let template = BagDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        edit_url: format!("/bags/{id}/edit"),
        bag: view,
        roaster_slug: roaster.slug.clone(),
        roast_slug: roast.slug.clone(),
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}

#[tracing::instrument(skip(state, _auth_user))]
pub(crate) async fn bag_edit_page(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    Path(id): Path<BagId>,
) -> Result<Response, StatusCode> {
    let bag = state
        .bag_repo
        .get_with_roast(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let roast_options = load_roast_options(&state).await.map_err(map_app_error)?;

    let template = BagEditTemplate {
        nav_active: "",
        is_authenticated: true,
        version_info: &crate::VERSION_INFO,
        id: bag.bag.id.to_string(),
        roast_id: bag.bag.roast_id.to_string(),
        roast_label: format!("{} ({})", bag.roast_name, bag.roaster_name),
        roast_date: bag
            .bag
            .roast_date
            .map(|d| d.to_string())
            .unwrap_or_default(),
        amount: bag.bag.amount,
        remaining: bag.bag.remaining,
        roast_options,
    };

    render_html(template).map(IntoResponse::into_response)
}
