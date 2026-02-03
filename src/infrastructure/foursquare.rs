use std::time::Duration;

use isocountry::CountryCode;
use serde::{Deserialize, Serialize};

use crate::application::errors::AppError;

pub const FOURSQUARE_SEARCH_URL: &str = "https://places-api.foursquare.com/places/search";
const USER_AGENT: &str = "Brewlog/1.0";
const MAX_RESULTS: &str = "15";
const RADIUS: &str = "5000";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const FIELDS: &str = "name,latitude,longitude,location,website,distance";
const API_VERSION: &str = "2025-06-17";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyCafe {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub country: String,
    pub website: Option<String>,
    pub distance_meters: u32,
}

/// Searches for places matching `query` near the given coordinates via Foursquare.
pub async fn search_nearby(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    lat: f64,
    lng: f64,
    query: &str,
) -> Result<Vec<NearbyCafe>, AppError> {
    let ll = format!("{lat},{lng}");

    let response = client
        .get(base_url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("X-Places-Api-Version", API_VERSION)
        .timeout(REQUEST_TIMEOUT)
        .query(&[("query", query), ("limit", MAX_RESULTS), ("fields", FIELDS)])
        .query(&[("ll", ll.as_str()), ("radius", RADIUS)])
        .send()
        .await
        .map_err(|e| AppError::unexpected(format!("Foursquare search failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "(unreadable body)".to_string());
        return Err(AppError::unexpected(format!(
            "Foursquare returned status {status}: {body}"
        )));
    }

    let result: FoursquareResponse = response
        .json()
        .await
        .map_err(|e| AppError::unexpected(format!("Failed to parse Foursquare response: {e}")))?;

    let cafes = result
        .results
        .into_iter()
        .filter_map(|place| {
            if place.name.is_empty() {
                return None;
            }

            let place_lat = place.latitude?;
            let place_lng = place.longitude?;
            let place_location = place.location.unwrap_or_default();

            let country = place_location
                .country
                .as_deref()
                .map(country_name)
                .unwrap_or_default();

            let distance = place
                .distance
                .unwrap_or_else(|| haversine_distance(lat, lng, place_lat, place_lng) as u32);

            let website = place.website.filter(|w| !w.trim().is_empty());

            Some(NearbyCafe {
                name: place.name,
                latitude: place_lat,
                longitude: place_lng,
                city: place_location.locality.unwrap_or_default(),
                country,
                website,
                distance_meters: distance,
            })
        })
        .collect();

    Ok(cafes)
}

/// Converts a 2-letter ISO 3166-1 alpha-2 country code to a full country name.
/// Falls back to the raw code if the lookup fails.
fn country_name(code: &str) -> String {
    // Override verbose ISO 3166-1 names with common short forms
    match code.to_ascii_uppercase().as_str() {
        "GB" => "United Kingdom".to_string(),
        "US" => "United States".to_string(),
        "KR" => "South Korea".to_string(),
        "KP" => "North Korea".to_string(),
        "TW" => "Taiwan".to_string(),
        "RU" => "Russia".to_string(),
        "IR" => "Iran".to_string(),
        "SY" => "Syria".to_string(),
        "VE" => "Venezuela".to_string(),
        "BO" => "Bolivia".to_string(),
        "TZ" => "Tanzania".to_string(),
        _ => CountryCode::for_alpha2_caseless(code)
            .map_or_else(|_| code.to_string(), |cc| cc.name().to_string()),
    }
}

/// Haversine distance in meters between two lat/lng points.
fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const R: f64 = 6_371_000.0; // Earth radius in meters

    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();

    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().asin();
    R * c
}

// --- Foursquare API types ---

#[derive(Debug, Deserialize)]
struct FoursquareResponse {
    results: Vec<FoursquarePlace>,
}

#[derive(Debug, Deserialize)]
struct FoursquarePlace {
    name: String,
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
    #[serde(default)]
    location: Option<FoursquareLocation>,
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    distance: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
struct FoursquareLocation {
    locality: Option<String>,
    country: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_london_to_paris() {
        // London (51.5074, -0.1278) to Paris (48.8566, 2.3522) ≈ 344 km
        let dist = haversine_distance(51.5074, -0.1278, 48.8566, 2.3522);
        let km = dist / 1000.0;
        assert!((km - 344.0).abs() < 5.0, "Expected ~344 km, got {km:.1} km");
    }

    #[test]
    fn haversine_same_point_is_zero() {
        let dist = haversine_distance(51.5, -0.1, 51.5, -0.1);
        assert!(dist.abs() < 0.01, "Expected 0, got {dist}");
    }

    #[test]
    fn parse_foursquare_search_response() {
        let json = r#"{
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
                }
            ]
        }"#;

        let response: FoursquareResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 2);

        let first = &response.results[0];
        assert_eq!(first.name, "Prufrock Coffee");
        assert_eq!(first.latitude, Some(51.5246));
        assert_eq!(
            first.location.as_ref().unwrap().locality.as_deref(),
            Some("London")
        );
        assert_eq!(
            first.website.as_deref(),
            Some("https://www.prufrockcoffee.com")
        );
        assert_eq!(first.distance, Some(2800));

        let second = &response.results[1];
        assert_eq!(second.name, "Department of Coffee");
        assert!(second.website.is_none());
    }

    #[test]
    fn parse_foursquare_result_without_location() {
        let json = r#"{
            "results": [{
                "name": "Café de Flore",
                "latitude": 48.8566,
                "longitude": 2.3522,
                "distance": 100
            }]
        }"#;

        let response: FoursquareResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].location.is_none());
        assert!(response.results[0].website.is_none());
    }

    #[test]
    fn country_code_to_name() {
        assert_eq!(country_name("GB"), "United Kingdom");
        assert_eq!(country_name("US"), "United States");
        assert_eq!(country_name("FR"), "France");
        // Unknown codes fall back to raw value
        assert_eq!(country_name("XX"), "XX");
    }
}
