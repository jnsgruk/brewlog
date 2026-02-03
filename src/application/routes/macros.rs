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
            axum::extract::State(state): axum::extract::State<crate::application::server::AppState>,
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
            axum::extract::State(state): axum::extract::State<crate::application::server::AppState>,
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
/// # Arguments
/// * `$fn_name` - Name of the generated handler function
/// * `$id_type` - Type of the ID path parameter (e.g., `RoasterId`)
/// * `$sort_key` - Sort key type for list requests (e.g., `RoasterSortKey`)
/// * `$repo_field` - Name of the repository field on `AppState` (e.g., `roaster_repo`)
/// * `$render_fragment` - Path to the fragment render function
///
/// # Example
/// ```ignore
/// define_delete_handler!(
///     delete_roaster,
///     RoasterId,
///     RoasterSortKey,
///     roaster_repo,
///     render_roaster_list_fragment
/// );
/// ```
macro_rules! define_delete_handler {
    ($fn_name:ident, $id_type:ty, $sort_key:ty, $repo_field:ident, $render_fragment:path) => {
        #[tracing::instrument(skip(state, _auth_user, headers, query))]
        pub(crate) async fn $fn_name(
            axum::extract::State(state): axum::extract::State<crate::application::server::AppState>,
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

            if crate::application::routes::support::is_datastar_request(&headers) {
                $render_fragment(state, request, search, true)
                    .await
                    .map_err(crate::application::errors::ApiError::from)
            } else {
                Ok(axum::http::StatusCode::NO_CONTENT.into_response())
            }
        }
    };
}

pub(super) use define_delete_handler;
pub(super) use define_enriched_get_handler;
pub(super) use define_get_handler;
