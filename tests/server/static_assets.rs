use crate::helpers::spawn_app;

macro_rules! define_static_asset_test {
    ($name:ident, $path:expr, $content_type:expr) => {
        #[tokio::test]
        async fn $name() {
            let app = spawn_app().await;
            let client = reqwest::Client::new();

            let response = client
                .get(app.page_url($path))
                .send()
                .await
                .expect("Failed to execute request");

            assert_eq!(response.status(), 200);
            assert_eq!(
                response
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok()),
                Some($content_type),
                "Wrong content-type for {}",
                $path
            );
            assert_eq!(
                response
                    .headers()
                    .get("cache-control")
                    .and_then(|v| v.to_str().ok()),
                Some("public, max-age=604800"),
                "Wrong cache-control for {}",
                $path
            );
        }
    };
}

define_static_asset_test!(
    styles_css,
    "/static/css/styles.css",
    "text/css; charset=utf-8"
);
define_static_asset_test!(
    webauthn_js,
    "/static/js/webauthn.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    location_js,
    "/static/js/location.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    image_utils_js,
    "/static/js/image-utils.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    photo_capture_js,
    "/static/js/components/photo-capture.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    searchable_select_js,
    "/static/js/components/searchable-select.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    chip_scroll_js,
    "/static/js/components/chip-scroll.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    world_map_js,
    "/static/js/components/world-map.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    donut_chart_js,
    "/static/js/components/donut-chart.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(
    image_upload_js,
    "/static/js/components/image-upload.js",
    "application/javascript; charset=utf-8"
);
define_static_asset_test!(favicon_light, "/static/favicon-light.svg", "image/svg+xml");
define_static_asset_test!(favicon_dark, "/static/favicon-dark.svg", "image/svg+xml");
define_static_asset_test!(og_image, "/static/og-image.png", "image/png");
define_static_asset_test!(app_icon_192, "/static/app-icon-192.png", "image/png");
define_static_asset_test!(app_icon_512, "/static/app-icon-512.png", "image/png");
define_static_asset_test!(
    site_webmanifest,
    "/static/site.webmanifest",
    "application/manifest+json; charset=utf-8"
);
