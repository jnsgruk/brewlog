use brewlog::infrastructure::osm::NearbyCafe;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app_with_nominatim_mock;

/// Canned Nominatim JSON for two results near London (51.5, -0.1).
fn nominatim_two_results() -> serde_json::Value {
    serde_json::json!([
        {
            "place_id": 123,
            "lat": "51.5246",
            "lon": "-0.1098",
            "display_name": "Prufrock Coffee, Leather Lane, London, England, United Kingdom",
            "namedetails": { "name": "Prufrock Coffee" },
            "address": {
                "cafe": "Prufrock Coffee",
                "road": "Leather Lane",
                "city": "London",
                "country": "United Kingdom"
            },
            "extratags": {
                "website": "https://www.prufrockcoffee.com"
            }
        },
        {
            "place_id": 456,
            "lat": "51.5200",
            "lon": "-0.1050",
            "display_name": "Department of Coffee, Leather Lane, London, England, United Kingdom",
            "namedetails": { "name": "Department of Coffee" },
            "address": {
                "city": "London",
                "country": "United Kingdom"
            }
        }
    ])
}

#[tokio::test]
async fn nearby_search_returns_results() {
    let app = spawn_app_with_nominatim_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("q", "coffee"))
        .and(query_param("format", "json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(nominatim_two_results()))
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

    let cafes: Vec<NearbyCafe> = response.json().await.expect("Failed to parse response");
    assert_eq!(cafes.len(), 2);

    assert_eq!(cafes[0].name, "Prufrock Coffee");
    assert_eq!(cafes[0].city, "London");
    assert_eq!(cafes[0].country, "United Kingdom");
    assert_eq!(
        cafes[0].website.as_deref(),
        Some("https://www.prufrockcoffee.com")
    );
    assert!(cafes[0].distance_meters > 0);

    assert_eq!(cafes[1].name, "Department of Coffee");
    assert!(cafes[1].website.is_none());
}

#[tokio::test]
async fn nearby_search_returns_empty_for_no_matches() {
    let app = spawn_app_with_nominatim_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
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

    let cafes: Vec<NearbyCafe> = response.json().await.expect("Failed to parse response");
    assert!(cafes.is_empty());
}

#[tokio::test]
async fn nearby_search_requires_authentication() {
    let app = spawn_app_with_nominatim_mock().await;

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
    let app = spawn_app_with_nominatim_mock().await;

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
    let app = spawn_app_with_nominatim_mock().await;

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
    let app = spawn_app_with_nominatim_mock().await;
    let mock_server = app.mock_server.as_ref().unwrap();

    Mock::given(method("GET"))
        .and(path("/search"))
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
