use crate::helpers::{
    TestApp, create_default_bag, create_default_cafe, create_default_gear, create_default_roast,
    create_default_roaster, post_form, spawn_app_with_auth,
};
use crate::test_macros::{define_form_create_tests, define_form_update_tests};

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
// Update setup functions: create entity + return (id, update form fields)
// ---------------------------------------------------------------------------

async fn roaster_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let roaster = create_default_roaster(app).await;
    (
        roaster.id.into_inner().to_string(),
        vec![("name".into(), "Updated Roasters".into())],
    )
}

async fn roast_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    (
        roast.id.into_inner().to_string(),
        vec![("name".into(), "Updated Roast Name".into())],
    )
}

async fn bag_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    let bag = create_default_bag(app, roast.id).await;
    (
        bag.id.into_inner().to_string(),
        vec![("amount".into(), "300".into())],
    )
}

async fn brew_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    let bag = create_default_bag(app, roast.id).await;
    let grinder = create_default_gear(app, "grinder", "Comandante", "C40 MK4").await;
    let brewer = create_default_gear(app, "brewer", "Hario", "V60").await;
    let brew: brewlog::domain::brews::Brew = crate::helpers::create_entity(
        app,
        "/brews",
        &brewlog::domain::brews::NewBrew {
            bag_id: bag.id,
            coffee_weight: 15.0,
            grinder_id: grinder.id,
            grind_setting: 24.0,
            brewer_id: brewer.id,
            filter_paper_id: None,
            water_volume: 250,
            water_temp: 92.0,
            quick_notes: Vec::new(),
            brew_time: None,
            created_at: None,
        },
    )
    .await;
    (
        brew.id.into_inner().to_string(),
        vec![("water_temp".into(), "94.0".into())],
    )
}

async fn cup_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let roaster = create_default_roaster(app).await;
    let roast = create_default_roast(app, roaster.id).await;
    let cafe = create_default_cafe(app).await;
    let cup: brewlog::domain::cups::Cup = crate::helpers::create_entity(
        app,
        "/cups",
        &brewlog::domain::cups::NewCup {
            roast_id: roast.id,
            cafe_id: cafe.id,
            created_at: None,
        },
    )
    .await;
    // Update the cafe to a different one
    let cafe2 = crate::helpers::create_cafe_with_payload(
        app,
        brewlog::domain::cafes::NewCafe {
            name: "Form Update Cafe".to_string(),
            city: "London".to_string(),
            country: "UK".to_string(),
            latitude: 51.5074,
            longitude: -0.1278,
            website: None,
            created_at: None,
        },
    )
    .await;
    (
        cup.id.into_inner().to_string(),
        vec![("cafe_id".into(), cafe2.id.into_inner().to_string())],
    )
}

async fn gear_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let gear = create_default_gear(app, "grinder", "Original", "Model").await;
    (
        gear.id.into_inner().to_string(),
        vec![("make".into(), "Updated Make".into())],
    )
}

async fn cafe_update_form(app: &TestApp) -> (String, Vec<(String, String)>) {
    let cafe = create_default_cafe(app).await;
    (
        cafe.id.into_inner().to_string(),
        vec![("name".into(), "Updated Cafe".into())],
    )
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
// Macro-generated tests: form update → 303 redirect + datastar variant
// ---------------------------------------------------------------------------

define_form_update_tests!(
    entity: roaster,
    api_path: "/roasters",
    redirect_prefix: "/roasters/",
    setup_and_form: roaster_update_form
);

define_form_update_tests!(
    entity: roast,
    api_path: "/roasts",
    redirect_prefix: "/roasters/",
    setup_and_form: roast_update_form
);

define_form_update_tests!(
    entity: bag,
    api_path: "/bags",
    redirect_prefix: "/bags/",
    setup_and_form: bag_update_form
);

define_form_update_tests!(
    entity: brew,
    api_path: "/brews",
    redirect_prefix: "/brews/",
    setup_and_form: brew_update_form
);

define_form_update_tests!(
    entity: cup,
    api_path: "/cups",
    redirect_prefix: "/cups/",
    setup_and_form: cup_update_form
);

define_form_update_tests!(
    entity: gear,
    api_path: "/gear",
    redirect_prefix: "/gear/",
    setup_and_form: gear_update_form
);

define_form_update_tests!(
    entity: cafe,
    api_path: "/cafes",
    redirect_prefix: "/cafes/",
    setup_and_form: cafe_update_form
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
