pub(crate) mod analytics;
pub(crate) mod auth;
pub(crate) mod coffee;
pub(crate) mod images;
pub(crate) mod macros;
pub(crate) mod system;

// Re-exports for backward compatibility
pub(crate) use analytics::stats;
pub(crate) use auth::{tokens, webauthn};
pub(crate) use coffee::{bags, brews, cafes, checkin, cups, gear, roasters, roasts, scan};
pub(crate) use system::{admin, backup, timeline};

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

use crate::application::state::AppState;

#[allow(clippy::too_many_lines)]
pub(super) fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route(
            "/roasters",
            get(roasters::list_roasters).post(roasters::create_roaster),
        )
        .route(
            "/roasters/{id}",
            get(roasters::get_roaster)
                .put(roasters::update_roaster)
                .delete(roasters::delete_roaster),
        )
        .route(
            "/roasts",
            get(roasts::list_roasts).post(roasts::create_roast),
        )
        .route(
            "/roasts/{id}",
            get(roasts::get_roast)
                .put(roasts::update_roast)
                .delete(roasts::delete_roast),
        )
        .route("/bags", get(bags::list_bags).post(bags::create_bag))
        .route(
            "/bags/{id}",
            get(bags::get_bag)
                .put(bags::update_bag)
                .delete(bags::delete_bag),
        )
        .route("/gear", get(gear::list_gear).post(gear::create_gear))
        .route(
            "/gear/{id}",
            get(gear::get_gear)
                .put(gear::update_gear)
                .delete(gear::delete_gear),
        )
        .route("/brews", get(brews::list_brews).post(brews::create_brew))
        .route(
            "/brews/{id}",
            get(brews::get_brew)
                .put(brews::update_brew)
                .delete(brews::delete_brew),
        )
        .route("/cafes", get(cafes::list_cafes).post(cafes::create_cafe))
        .route(
            "/cafes/{id}",
            get(cafes::get_cafe)
                .put(cafes::update_cafe)
                .delete(cafes::delete_cafe),
        )
        .route("/nearby-cafes", get(cafes::nearby_cafes))
        .route("/extract-roaster", post(roasters::extract_roaster))
        .route("/extract-roast", post(roasts::extract_roast_info))
        .route(
            "/extract-bag-scan",
            post(scan::extract_bag_scan).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route(
            "/scan",
            post(scan::submit_scan).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/check-in", post(checkin::submit_checkin))
        .route("/cups", get(cups::list_cups).post(cups::create_cup))
        .route(
            "/cups/{id}",
            get(cups::get_cup)
                .put(cups::update_cup)
                .delete(cups::delete_cup),
        )
        .route(
            "/tokens",
            post(tokens::create_token).get(tokens::list_tokens),
        )
        .route("/tokens/{id}/revoke", post(tokens::revoke_token))
        .route("/passkeys", get(admin::list_passkeys))
        .route(
            "/passkeys/{id}",
            axum::routing::delete(admin::delete_passkey),
        )
        .route("/backup", get(backup::export_backup))
        .route(
            "/backup/restore",
            post(backup::restore_backup).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .route("/backup/reset", post(backup::reset_database))
        .route("/stats/recompute", post(stats::recompute_stats))
        .route("/timeline/rebuild", post(timeline::rebuild_timeline))
        .route(
            "/{entity_type}/{id}/image",
            get(images::get_image)
                .put(images::upload_image)
                .delete(images::delete_image)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/{entity_type}/{id}/thumbnail", get(images::get_thumbnail))
}

pub(super) fn webauthn_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/register/start", post(webauthn::register_start))
        .route("/register/finish", post(webauthn::register_finish))
        .route("/auth/start", get(webauthn::auth_start))
        .route("/auth/finish", post(webauthn::auth_finish))
        .route("/passkey/start", post(webauthn::passkey_add_start))
        .route("/passkey/finish", post(webauthn::passkey_add_finish))
        .route(
            "/auth/discoverable/start",
            get(webauthn::discoverable_auth_start),
        )
        .route(
            "/auth/discoverable/finish",
            post(webauthn::discoverable_auth_finish),
        )
}
