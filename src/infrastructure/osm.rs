use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::application::errors::AppError;

pub const NOMINATIM_SEARCH_URL: &str = "https://nominatim.openstreetmap.org/search";
const USER_AGENT: &str = "Brewlog/1.0";
const MAX_RESULTS: &str = "8";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Viewbox half-size in degrees (~11 km at equator, tighter at higher latitudes).
const VIEWBOX_DELTA: f64 = 0.1;

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

/// Searches for places matching `query` near the given coordinates via Nominatim.
/// Results are biased towards (but not restricted to) the user's location.
pub async fn search_nearby(
    client: &reqwest::Client,
    base_url: &str,
    lat: f64,
    lng: f64,
    query: &str,
) -> Result<Vec<NearbyCafe>, AppError> {
    let viewbox = format!(
        "{},{},{},{}",
        lng - VIEWBOX_DELTA,
        lat + VIEWBOX_DELTA,
        lng + VIEWBOX_DELTA,
        lat - VIEWBOX_DELTA,
    );

    let response = client
        .get(base_url)
        .header("User-Agent", USER_AGENT)
        .timeout(REQUEST_TIMEOUT)
        .query(&[
            ("q", query),
            ("format", "json"),
            ("limit", MAX_RESULTS),
            ("addressdetails", "1"),
            ("namedetails", "1"),
            ("extratags", "1"),
            ("viewbox", &viewbox),
            ("bounded", "0"),
        ])
        .send()
        .await
        .map_err(|e| AppError::unexpected(format!("Nominatim search failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::unexpected(format!(
            "Nominatim returned status {}",
            response.status()
        )));
    }

    let results: Vec<NominatimSearchResult> = response
        .json()
        .await
        .map_err(|e| AppError::unexpected(format!("Failed to parse Nominatim response: {e}")))?;

    let cafes = results
        .into_iter()
        .filter_map(|r| {
            let result_lat: f64 = r.lat.parse().ok()?;
            let result_lng: f64 = r.lon.parse().ok()?;

            let name = r.namedetails.and_then(|nd| nd.name).unwrap_or_else(|| {
                // Fall back to text before first comma in display_name
                r.display_name
                    .split(',')
                    .next()
                    .unwrap_or(&r.display_name)
                    .trim()
                    .to_string()
            });

            if name.is_empty() {
                return None;
            }

            let address = r.address.unwrap_or_default();

            let city = address
                .city
                .or(address.town)
                .or(address.village)
                .unwrap_or_default();

            let website = r
                .extratags
                .and_then(|tags| {
                    tags.website
                        .or(tags.contact_website)
                        .or(tags.url)
                        .or(tags.contact_url)
                        .or(tags.brand_website)
                })
                .filter(|w| !w.trim().is_empty());

            let distance = haversine_distance(lat, lng, result_lat, result_lng);

            Some(NearbyCafe {
                name,
                latitude: result_lat,
                longitude: result_lng,
                city,
                country: address.country.unwrap_or_default(),
                website,
                distance_meters: distance as u32,
            })
        })
        .collect();

    Ok(cafes)
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

// --- Nominatim types ---

#[derive(Debug, Deserialize)]
struct NominatimSearchResult {
    lat: String,
    lon: String,
    display_name: String,
    #[serde(default)]
    namedetails: Option<NominatimNameDetails>,
    #[serde(default)]
    address: Option<NominatimAddress>,
    #[serde(default)]
    extratags: Option<NominatimExtraTags>,
}

#[derive(Debug, Deserialize)]
struct NominatimNameDetails {
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NominatimExtraTags {
    website: Option<String>,
    #[serde(rename = "contact:website")]
    contact_website: Option<String>,
    url: Option<String>,
    #[serde(rename = "contact:url")]
    contact_url: Option<String>,
    #[serde(rename = "brand:website")]
    brand_website: Option<String>,
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
    fn parse_nominatim_search_response() {
        let json = r#"[
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
                    "state": "England",
                    "country": "United Kingdom",
                    "country_code": "gb"
                },
                "extratags": {
                    "website": "https://www.prufrockcoffee.com"
                }
            },
            {
                "place_id": 456,
                "lat": "51.4543",
                "lon": "-2.5930",
                "display_name": "Full Court Press, Broad Street, Bristol, England, United Kingdom",
                "namedetails": { "name": "Full Court Press" },
                "address": {
                    "town": "Bristol",
                    "country": "United Kingdom"
                }
            }
        ]"#;

        let results: Vec<NominatimSearchResult> = serde_json::from_str(json).unwrap();
        assert_eq!(results.len(), 2);

        let first = &results[0];
        assert_eq!(first.lat, "51.5246");
        assert_eq!(
            first.namedetails.as_ref().unwrap().name.as_deref(),
            Some("Prufrock Coffee")
        );
        assert_eq!(
            first.address.as_ref().unwrap().city.as_deref(),
            Some("London")
        );
        assert_eq!(
            first.extratags.as_ref().unwrap().website.as_deref(),
            Some("https://www.prufrockcoffee.com")
        );

        // Second result uses town fallback, no extratags
        let second = &results[1];
        assert!(second.address.as_ref().unwrap().city.is_none());
        assert_eq!(
            second.address.as_ref().unwrap().town.as_deref(),
            Some("Bristol")
        );
        assert!(second.extratags.is_none());
    }

    #[test]
    fn parse_nominatim_result_without_namedetails() {
        let json = r#"[{
            "place_id": 789,
            "lat": "48.8566",
            "lon": "2.3522",
            "display_name": "Café de Flore, Boulevard Saint-Germain, Paris, France",
            "address": {
                "city": "Paris",
                "country": "France"
            }
        }]"#;

        let results: Vec<NominatimSearchResult> = serde_json::from_str(json).unwrap();
        assert_eq!(results.len(), 1);

        // Without namedetails, should fall back to display_name prefix
        assert!(results[0].namedetails.is_none());
        assert_eq!(
            results[0].display_name.split(',').next().unwrap().trim(),
            "Café de Flore"
        );
    }

    #[test]
    fn website_fallback_chain() {
        // url tag should be used when website and contact:website are absent
        let json = r#"[{
            "place_id": 101,
            "lat": "51.5",
            "lon": "-0.1",
            "display_name": "Test Cafe, London, UK",
            "namedetails": { "name": "Test Cafe" },
            "address": { "city": "London", "country": "United Kingdom" },
            "extratags": { "url": "https://testcafe.example.com" }
        }]"#;

        let results: Vec<NominatimSearchResult> = serde_json::from_str(json).unwrap();
        let tags = results[0].extratags.as_ref().unwrap();
        assert!(tags.website.is_none());
        assert!(tags.contact_website.is_none());
        assert_eq!(tags.url.as_deref(), Some("https://testcafe.example.com"));
    }
}
