use crate::helpers::{
    TestApp, create_default_bag, create_default_cafe, create_default_gear, create_default_roast,
    create_default_roaster, post_form, spawn_app_with_auth,
};
use crate::test_macros::define_form_create_tests;

// ---------------------------------------------------------------------------
// Setup functions: create prerequisites + return form fields
// ---------------------------------------------------------------------------

async fn roaster_form_fields(_app: &TestApp) -> Vec<(String, String)> {
    vec![
        ("name".into(), "Form Roasters".into()),
        ("country".into(), "UK".into()),
    ]
}

async fn roast_form_fields(app: &TestApp) -> Vec<(String, String)> {
    let roaster = create_default_roaster(app).await;
    vec![
        ("roaster_id".into(), roaster.id.into_inner().to_string()),
        ("name".into(), "Form Roast".into()),
        ("origin".into(), "Ethiopia".into()),
        ("region".into(), "Yirgacheffe".into()),
        ("producer".into(), "Coop".into()),
        ("tasting_notes".into(), "Blueberry, Jasmine".into()),
        ("process".into(), "Washed".into()),
    ]
}

async fn bag_form_fields(app: &TestApp) -> Vec<(String, String)> {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    vec![
        ("roast_id".into(), roast.id.into_inner().to_string()),
        ("roast_date".into(), "2023-06-15".into()),
        ("amount".into(), "250".into()),
    ]
}

async fn brew_form_fields(app: &TestApp) -> Vec<(String, String)> {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    let bag = create_default_bag(app, roast.id).await;
    let grinder = create_default_gear(app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(app, "brewer", "Hario", "V60").await;
    vec![
        ("bag_id".into(), bag.id.into_inner().to_string()),
        ("coffee_weight".into(), "15.0".into()),
        ("grinder_id".into(), grinder.id.into_inner().to_string()),
        ("grind_setting".into(), "24.0".into()),
        ("brewer_id".into(), brewer.id.into_inner().to_string()),
        ("water_volume".into(), "250".into()),
        ("water_temp".into(), "96.0".into()),
    ]
}

async fn cup_form_fields(app: &TestApp) -> Vec<(String, String)> {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    let cafe = create_default_cafe(app).await;
    vec![
        ("roast_id".into(), roast.id.into_inner().to_string()),
        ("cafe_id".into(), cafe.id.into_inner().to_string()),
    ]
}

async fn gear_form_fields(_app: &TestApp) -> Vec<(String, String)> {
    vec![
        ("category".into(), "grinder".into()),
        ("make".into(), "Comandante".into()),
        ("model".into(), "C40 MK4".into()),
    ]
}

async fn cafe_form_fields(_app: &TestApp) -> Vec<(String, String)> {
    vec![
        ("name".into(), "Form Cafe".into()),
        ("city".into(), "London".into()),
        ("country".into(), "UK".into()),
        ("latitude".into(), "51.5074".into()),
        ("longitude".into(), "-0.1278".into()),
    ]
}

async fn checkin_form_fields(app: &TestApp) -> Vec<(String, String)> {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    let cafe = create_default_cafe(app).await;
    vec![
        ("cafe_id".into(), cafe.id.into_inner().to_string()),
        ("roast_id".into(), roast.id.into_inner().to_string()),
    ]
}

// ---------------------------------------------------------------------------
// Macro-generated tests: form create → 303 redirect + datastar variant
// ---------------------------------------------------------------------------

define_form_create_tests!(
    entity: roaster,
    api_path: "/roasters",
    redirect_prefix: "/roasters/",
    setup_and_form: roaster_form_fields
);

define_form_create_tests!(
    entity: roast,
    api_path: "/roasts",
    redirect_prefix: "/roasters/",
    setup_and_form: roast_form_fields
);

define_form_create_tests!(
    entity: bag,
    api_path: "/bags",
    redirect_prefix: "/bags/",
    setup_and_form: bag_form_fields
);

define_form_create_tests!(
    entity: brew,
    api_path: "/brews",
    redirect_prefix: "/brews/",
    setup_and_form: brew_form_fields
);

define_form_create_tests!(
    entity: cup,
    api_path: "/cups",
    redirect_prefix: "/cups/",
    setup_and_form: cup_form_fields
);

define_form_create_tests!(
    entity: gear,
    api_path: "/gear",
    redirect_prefix: "/gear/",
    setup_and_form: gear_form_fields
);

define_form_create_tests!(
    entity: cafe,
    api_path: "/cafes",
    redirect_prefix: "/cafes/",
    setup_and_form: cafe_form_fields
);

define_form_create_tests!(
    entity: checkin,
    api_path: "/check-in",
    redirect_prefix: "/cups/",
    setup_and_form: checkin_form_fields
);

// ---------------------------------------------------------------------------
// Hand-written tests: form-specific parsing edge cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn brew_form_with_empty_filter_paper_and_quick_notes() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;
    let bag = create_default_bag(&app, roast.id).await;
    let grinder = create_default_gear(&app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(&app, "brewer", "Hario", "V60").await;

    let form_fields = vec![
        ("bag_id", bag.id.into_inner().to_string()),
        ("coffee_weight", "15.0".into()),
        ("grinder_id", grinder.id.into_inner().to_string()),
        ("grind_setting", "24.0".into()),
        ("brewer_id", brewer.id.into_inner().to_string()),
        ("water_volume", "250".into()),
        ("water_temp", "96.0".into()),
        ("filter_paper_id", String::new()), // empty string → None
        ("quick_notes", "good,too-fast".into()), // comma-separated → vec
    ];

    let response = post_form(&app, "/brews", &form_fields).await;
    assert_eq!(response.status(), 303);
}

#[tokio::test]
async fn roast_form_with_comma_separated_tasting_notes() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;

    let form_fields = vec![
        ("roaster_id", roaster.id.into_inner().to_string()),
        ("name", "Tasting Test".into()),
        ("origin", "Kenya".into()),
        ("region", "Nyeri".into()),
        ("producer", "Smallholder".into()),
        ("tasting_notes", "Blackcurrant, Tomato, Brown Sugar".into()),
        ("process", "Natural".into()),
    ];

    let response = post_form(&app, "/roasts", &form_fields).await;
    assert_eq!(response.status(), 303);

    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .expect("missing Location header");
    assert!(location.starts_with("/roasters/"));
}

#[tokio::test]
async fn bag_form_with_empty_roast_date() {
    let app = spawn_app_with_auth().await;
    let roaster = create_default_roaster(&app).await;
    let roast = create_default_roast(&app, roaster.id).await;

    let form_fields = vec![
        ("roast_id", roast.id.into_inner().to_string()),
        ("roast_date", String::new()), // empty string → None
        ("amount", "250".into()),
    ];

    let response = post_form(&app, "/bags", &form_fields).await;
    assert_eq!(response.status(), 303);
}
