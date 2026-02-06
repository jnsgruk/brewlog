pub(crate) mod account;
pub(crate) mod backup;
pub(crate) mod bags;
pub(crate) mod brews;
pub(crate) mod cafes;
pub(crate) mod checkin;
pub(crate) mod cups;
pub(crate) mod gear;
mod macros;
pub(crate) mod roasters;
pub(crate) mod roasts;
pub(crate) mod scan;
pub(crate) mod tokens;
pub(crate) mod webauthn;

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
            "/roasters/:id",
            get(roasters::get_roaster)
                .put(roasters::update_roaster)
                .delete(roasters::delete_roaster),
        )
        .route(
            "/roasts",
            get(roasts::list_roasts).post(roasts::create_roast),
        )
        .route(
            "/roasts/:id",
            get(roasts::get_roast)
                .put(roasts::update_roast)
                .delete(roasts::delete_roast),
        )
        .route("/bags", get(bags::list_bags).post(bags::create_bag))
        .route(
            "/bags/:id",
            get(bags::get_bag)
                .put(bags::update_bag)
                .delete(bags::delete_bag),
        )
        .route("/gear", get(gear::list_gear).post(gear::create_gear))
        .route(
            "/gear/:id",
            get(gear::get_gear)
                .put(gear::update_gear)
                .delete(gear::delete_gear),
        )
        .route("/brews", get(brews::list_brews).post(brews::create_brew))
        .route(
            "/brews/:id",
            get(brews::get_brew).delete(brews::delete_brew),
        )
        .route("/cafes", get(cafes::list_cafes).post(cafes::create_cafe))
        .route(
            "/cafes/:id",
            get(cafes::get_cafe)
                .put(cafes::update_cafe)
                .delete(cafes::delete_cafe),
        )
        .route("/nearby-cafes", get(cafes::nearby_cafes))
        .route("/extract-roaster", post(roasters::extract_roaster))
        .route("/extract-roast", post(roasts::extract_roast_info))
        .route("/extract-bag-scan", post(scan::extract_bag_scan))
        .route("/scan", post(scan::submit_scan))
        .route("/check-in", post(checkin::submit_checkin))
        .route("/cups", get(cups::list_cups).post(cups::create_cup))
        .route("/cups/:id", get(cups::get_cup).delete(cups::delete_cup))
        .route(
            "/tokens",
            post(tokens::create_token).get(tokens::list_tokens),
        )
        .route("/tokens/:id/revoke", post(tokens::revoke_token))
        .route("/passkeys", get(account::list_passkeys))
        .route(
            "/passkeys/:id",
            axum::routing::delete(account::delete_passkey),
        )
        .route("/backup", get(backup::export_backup))
        .route(
            "/backup/restore",
            post(backup::restore_backup).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
}

pub(super) fn webauthn_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/register/start", post(webauthn::register_start))
        .route("/register/finish", post(webauthn::register_finish))
        .route("/auth/start", get(webauthn::auth_start))
        .route("/auth/finish", post(webauthn::auth_finish))
        .route("/passkey/start", post(webauthn::passkey_add_start))
        .route("/passkey/finish", post(webauthn::passkey_add_finish))
}
