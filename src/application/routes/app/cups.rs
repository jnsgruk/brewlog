use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tower_cookies::Cookies;

use crate::application::errors::map_app_error;
use crate::application::routes::api::images::resolve_image_url;
use crate::application::routes::render_html;
use crate::application::state::AppState;
use crate::domain::ids::CupId;
use crate::presentation::web::templates::CupDetailTemplate;
use crate::presentation::web::views::CupDetailView;

#[tracing::instrument(skip(state, cookies))]
pub(crate) async fn cup_detail_page(
    State(state): State<AppState>,
    cookies: Cookies,
    Path(id): Path<CupId>,
) -> Result<Response, StatusCode> {
    let is_authenticated = crate::application::routes::is_authenticated(&state, &cookies).await;

    let cup_details = state
        .cup_repo
        .get_with_details(id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let (roast, cafe) = tokio::try_join!(
        async {
            state
                .roast_repo
                .get(cup_details.cup.roast_id)
                .await
                .map_err(|e| map_app_error(e.into()))
        },
        async {
            state
                .cafe_repo
                .get(cup_details.cup.cafe_id)
                .await
                .map_err(|e| map_app_error(e.into()))
        },
    )?;

    let roaster = state
        .roaster_repo
        .get(roast.roaster_id)
        .await
        .map_err(|e| map_app_error(e.into()))?;

    let image_url = resolve_image_url(&state, "cup", i64::from(id))
        .await
        .or(resolve_image_url(&state, "cafe", i64::from(cafe.id)).await)
        .or(resolve_image_url(&state, "roast", i64::from(roast.id)).await);

    let view = CupDetailView::from_parts(cup_details, &roast, &roaster, &cafe);

    let template = CupDetailTemplate {
        nav_active: "",
        is_authenticated,
        version_info: &crate::VERSION_INFO,
        base_url: crate::base_url(),
        edit_url: format!("/cups/{id}/edit"),
        cup: view,
        roaster_slug: roaster.slug.clone(),
        roast_slug: roast.slug.clone(),
        cafe_slug: cafe.slug.clone(),
        image_url,
    };

    render_html(template).map(IntoResponse::into_response)
}
