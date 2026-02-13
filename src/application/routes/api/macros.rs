/// Generates a GET-by-ID handler that retrieves an entity from a repository.
///
/// # Arguments
/// * `$fn_name` - Name of the generated handler function
/// * `$id_type` - Type of the ID path parameter (e.g., `RoasterId`)
/// * `$entity_type` - Type of the entity returned as JSON (e.g., `Roaster`)
/// * `$repo_field` - Name of the repository field on `AppState` (e.g., `roaster_repo`)
///
/// # Example
/// ```ignore
/// define_get_handler!(get_roaster, RoasterId, Roaster, roaster_repo);
/// ```
macro_rules! define_get_handler {
    ($fn_name:ident, $id_type:ty, $entity_type:ty, $repo_field:ident) => {
        #[tracing::instrument(skip(state))]
        pub(crate) async fn $fn_name(
            axum::extract::State(state): axum::extract::State<crate::application::state::AppState>,
            axum::extract::Path(id): axum::extract::Path<$id_type>,
        ) -> Result<axum::Json<$entity_type>, crate::application::errors::ApiError> {
            let entity = state
                .$repo_field
                .get(id)
                .await
                .map_err(crate::application::errors::AppError::from)?;
            Ok(axum::Json(entity))
        }
    };
}

/// Generates a GET-by-ID handler that retrieves an enriched entity using a custom method.
///
/// # Arguments
/// * `$fn_name` - Name of the generated handler function
/// * `$id_type` - Type of the ID path parameter (e.g., `RoastId`)
/// * `$entity_type` - Type of the enriched entity returned as JSON (e.g., `RoastWithRoaster`)
/// * `$repo_field` - Name of the repository field on `AppState` (e.g., `roast_repo`)
/// * `$method` - Name of the repository method to call (e.g., `get_with_roaster`)
///
/// # Example
/// ```ignore
/// define_enriched_get_handler!(get_roast, RoastId, RoastWithRoaster, roast_repo, get_with_roaster);
/// ```
macro_rules! define_enriched_get_handler {
    ($fn_name:ident, $id_type:ty, $entity_type:ty, $repo_field:ident, $method:ident) => {
        #[tracing::instrument(skip(state))]
        pub(crate) async fn $fn_name(
            axum::extract::State(state): axum::extract::State<crate::application::state::AppState>,
            axum::extract::Path(id): axum::extract::Path<$id_type>,
        ) -> Result<axum::Json<$entity_type>, crate::application::errors::ApiError> {
            let entity = state
                .$repo_field
                .$method(id)
                .await
                .map_err(crate::application::errors::AppError::from)?;
            Ok(axum::Json(entity))
        }
    };
}

/// Generates a DELETE handler with Datastar fragment re-rendering support.
///
/// When a Datastar request arrives from the data/list page (detected via referer
/// containing `$referer_match`), the handler re-renders the list fragment. When the
/// request comes from elsewhere (e.g. a detail page), it returns a redirect script
/// pointing at `$redirect_url`. Non-Datastar requests get a 204 No Content.
///
/// # Arguments
/// * `$fn_name` - Name of the generated handler function
/// * `$id_type` - Type of the ID path parameter (e.g., `RoasterId`)
/// * `$sort_key` - Sort key type for list requests (e.g., `RoasterSortKey`)
/// * `$repo_field` - Name of the repository field on `AppState` (e.g., `roaster_repo`)
/// * `$render_fragment` - Path to the fragment render function
/// * `$referer_match` - String to look for in `Referer` header (e.g., `"type=roasters"`)
/// * `$redirect_url` - URL for the redirect script (e.g., `"/data?type=roasters"`)
///
/// # Example
/// ```ignore
/// define_delete_handler!(
///     delete_roaster,
///     RoasterId,
///     RoasterSortKey,
///     roaster_repo,
///     render_roaster_list_fragment,
///     "type=roasters",
///     "/data?type=roasters"
/// );
/// ```
macro_rules! define_delete_handler {
    ($fn_name:ident, $id_type:ty, $sort_key:ty, $repo_field:ident, $render_fragment:path, $referer_match:literal, $redirect_url:literal) => {
        define_delete_handler!(@inner $fn_name, $id_type, $sort_key, $repo_field, $render_fragment, $referer_match, $redirect_url, None);
    };
    ($fn_name:ident, $id_type:ty, $sort_key:ty, $repo_field:ident, $render_fragment:path, $referer_match:literal, $redirect_url:literal, image_type: $image_type:expr) => {
        define_delete_handler!(@inner $fn_name, $id_type, $sort_key, $repo_field, $render_fragment, $referer_match, $redirect_url, Some($image_type));
    };
    (@inner $fn_name:ident, $id_type:ty, $sort_key:ty, $repo_field:ident, $render_fragment:path, $referer_match:literal, $redirect_url:literal, $image_type:expr) => {
        #[tracing::instrument(skip(state, _auth_user, headers, query))]
        pub(crate) async fn $fn_name(
            axum::extract::State(state): axum::extract::State<crate::application::state::AppState>,
            _auth_user: crate::application::auth::AuthenticatedUser,
            headers: axum::http::HeaderMap,
            axum::extract::Path(id): axum::extract::Path<$id_type>,
            axum::extract::Query(query): axum::extract::Query<
                crate::application::routes::support::ListQuery,
            >,
        ) -> Result<axum::response::Response, crate::application::errors::ApiError> {
            let (request, search) = query.into_request_and_search::<$sort_key>();
            state
                .$repo_field
                .delete(id)
                .await
                .map_err(crate::application::errors::AppError::from)?;

            if let Some(img_type) = $image_type {
                if let Err(err) = state.image_repo.delete(img_type, i64::from(id)).await {
                    tracing::warn!(%id, error = %err, "failed to delete entity image");
                }
            }

            tracing::info!(%id, "entity deleted");
            state.stats_invalidator.invalidate();

            if crate::application::routes::support::is_datastar_request(&headers) {
                let from_data_page = headers
                    .get("referer")
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|r| r.contains($referer_match));

                if from_data_page {
                    $render_fragment(state, request, search, true)
                        .await
                        .map_err(crate::application::errors::ApiError::from)
                } else {
                    crate::application::routes::support::render_redirect_script($redirect_url)
                        .map_err(crate::application::errors::ApiError::from)
                }
            } else {
                Ok(axum::http::StatusCode::NO_CONTENT.into_response())
            }
        }
    };
}

/// Generates a list-fragment renderer for Datastar partial updates.
///
/// Produces a function that loads a page via `$loader`, builds the given list
/// template, and returns it as a Datastar fragment targeting `$selector`.
///
/// # Arguments
/// * `$fn_name`  - Name of the generated function
/// * `$sort_key` - Sort key type (e.g., `RoasterSortKey`)
/// * `$loader`   - Page-loader function returning `(Paginated<V>, ListNavigator<K>)`
/// * `$template { $field }` - List template type and its items field name
/// * `$selector` - CSS selector for Datastar patching (e.g., `"#roaster-list"`)
///
/// # Example
/// ```ignore
/// define_list_fragment_renderer!(
///     render_roaster_list_fragment,
///     RoasterSortKey,
///     load_roaster_page,
///     RoasterListTemplate { roasters },
///     "#roaster-list"
/// );
/// ```
macro_rules! define_list_fragment_renderer {
    ($fn_name:ident, $sort_key:ty, $loader:ident, $template:ident { $field:ident }, $selector:literal) => {
        async fn $fn_name(
            state: crate::application::state::AppState,
            request: crate::domain::listing::ListRequest<$sort_key>,
            search: Option<String>,
            is_authenticated: bool,
        ) -> Result<axum::response::Response, crate::application::errors::AppError> {
            let (items, navigator) = $loader(&state, request, search.as_deref()).await?;
            let template = $template {
                is_authenticated,
                $field: items,
                navigator,
            };
            crate::application::routes::support::render_fragment(template, $selector)
        }
    };
}

pub(crate) use define_delete_handler;
pub(crate) use define_enriched_get_handler;
pub(crate) use define_get_handler;
pub(crate) use define_list_fragment_renderer;
