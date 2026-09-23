use brewlog::domain::nearby_cafes::NearbyCafeResult;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app_with_foursquare_mock;

/// Deliberately unsorted Foursquare results near London (51.5, -0.1).
fn foursquare_results() -> serde_json::Value {
    serde_json::json!({
        "results": [
            {
                "name": "Prufrock Coffee",
                "latitude": 51.5246,
                "longitude": -0.1098,
                "location": {
                    "locality": "London",
                    "country": "GB"
                },
                "website": "https://www.prufrockcoffee.com",
                "distance": 2800
            },
            {
                "name": "Department of Coffee",
                "latitude": 51.5200,
                "longitude": -0.1050,
                "location": {
                    "locality": "London",
                    "country": "GB"
                },
                "distance": 2500
            },
            {
                "name": "Center Cafe",
                "latitude": 51.5,
                "longitude": -0.1
            },
            {
                "name": "Nearby Cafe",
                "latitude": 51.5001,
                "longitude": -0.1
            }
        ]
    })
}

#[tokio::test]
async fn nearby_search_returns_results() {
    let app = spawn_app_with_foursquare_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/places/search"))
        .and(query_param("query", "coffee"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(foursquare_results()))
        .expect(1)
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .query(&[("lat", "51.5"), ("lng", "-0.1"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let cafes: Vec<NearbyCafeResult> = response.json().await.expect("Failed to parse response");
    assert_eq!(cafes.len(), 4);
    assert_eq!(cafes[0].name, "Center Cafe");
    assert_eq!(cafes[0].distance_meters, Some(0));
    assert_eq!(cafes[1].name, "Nearby Cafe");
    assert!(cafes[1].distance_meters.unwrap() > 0);
    assert_eq!(cafes[2].name, "Department of Coffee");
    assert_eq!(cafes[2].distance_meters, Some(2500));
    assert_eq!(cafes[3].name, "Prufrock Coffee");
    assert_eq!(cafes[3].city, "London");
    assert_eq!(cafes[3].country, "United Kingdom");
    assert_eq!(
        cafes[3].website.as_deref(),
        Some("https://www.prufrockcoffee.com")
    );
    assert_eq!(cafes[3].distance_meters, Some(2800));
}

#[tokio::test]
async fn nearby_search_fragment_shows_closest_cafe_first() {
    let app = spawn_app_with_foursquare_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/places/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(foursquare_results()))
        .expect(1)
        .mount(mock_server)
        .await;

    let response = reqwest::Client::new()
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .query(&[("lat", "51.5"), ("lng", "-0.1"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("Failed to read fragment");
    let positions: Vec<_> = [
        "Center Cafe",
        "Nearby Cafe",
        "Department of Coffee",
        "Prufrock Coffee",
    ]
    .iter()
    .map(|name| body.find(&format!("data-cafe-name=\"{name}\"")).unwrap())
    .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[tokio::test]
async fn nearby_search_returns_empty_for_no_matches() {
    let app = spawn_app_with_foursquare_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/places/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"results": []})))
        .expect(1)
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .query(&[("lat", "51.5"), ("lng", "-0.1"), ("q", "nonexistent")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let cafes: Vec<NearbyCafeResult> = response.json().await.expect("Failed to parse response");
    assert!(cafes.is_empty());
}

#[tokio::test]
async fn nearby_search_requires_authentication() {
    let app = spawn_app_with_foursquare_mock().await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .query(&[("lat", "51.5"), ("lng", "-0.1"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn nearby_search_rejects_short_query() {
    let app = spawn_app_with_foursquare_mock().await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .query(&[("lat", "51.5"), ("lng", "-0.1"), ("q", "a")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn nearby_search_rejects_invalid_coordinates() {
    let app = spawn_app_with_foursquare_mock().await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .query(&[("lat", "999"), ("lng", "-0.1"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn nearby_search_returns_500_on_upstream_failure() {
    let app = spawn_app_with_foursquare_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/places/search"))
        .respond_with(ResponseTemplate::new(503))
        .expect(1)
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .query(&[("lat", "51.5"), ("lng", "-0.1"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 500);
}

#[tokio::test]
async fn nearby_search_with_near_param() {
    let app = spawn_app_with_foursquare_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/places/search"))
        .and(query_param("query", "coffee"))
        .and(query_param("near", "London"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(foursquare_results()))
        .expect(2)
        .mount(mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .query(&[("near", "London"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);

    let cafes: Vec<NearbyCafeResult> = response.json().await.expect("Failed to parse response");
    assert_eq!(cafes.len(), 4);
    assert_eq!(cafes[0].name, "Department of Coffee");
    assert_eq!(cafes[0].distance_meters, Some(2500));
    assert_eq!(cafes[1].name, "Prufrock Coffee");
    assert_eq!(cafes[1].distance_meters, Some(2800));
    assert_eq!(cafes[2].name, "Center Cafe");
    assert_eq!(cafes[2].distance_meters, None);
    assert_eq!(cafes[3].name, "Nearby Cafe");
    assert_eq!(cafes[3].distance_meters, None);

    let response = client
        .get(app.api_url("/nearby-cafes"))
        .bearer_auth(app.auth_token.as_ref().unwrap())
        .header("datastar-request", "true")
        .query(&[("near", "London"), ("q", "coffee")])
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), 200);
    let body = response.text().await.expect("Failed to read fragment");
    let positions: Vec<_> = [
        "Department of Coffee",
        "Prufrock Coffee",
        "Center Cafe",
        "Nearby Cafe",
    ]
    .iter()
    .map(|name| body.find(&format!("data-cafe-name=\"{name}\"")).unwrap())
    .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!body.contains("&middot; 0 m"));
}
