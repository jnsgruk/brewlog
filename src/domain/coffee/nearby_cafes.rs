use serde::{Deserialize, Serialize};

/// A nearby cafe result from a location-based search.
///
/// This is a domain-level representation that decouples the presentation
/// layer from any specific third-party API (e.g. Foursquare).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyCafeResult {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub city: String,
    pub country: String,
    pub website: Option<String>,
    pub distance_meters: u32,
}
