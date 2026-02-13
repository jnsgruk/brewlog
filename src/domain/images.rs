use std::fmt;

use serde::Deserialize;

use crate::domain::entity_type::EntityType;

/// An image associated with an entity (roaster, roast, gear, or cafe).
pub struct EntityImage {
    pub entity_type: EntityType,
    pub entity_id: i64,
    pub content_type: String,
    pub image_data: Vec<u8>,
    pub thumbnail_data: Vec<u8>,
}

/// Wrapper for image data URLs that redacts content in `Debug` output,
/// allowing payloads to be traced without logging raw base64 image data.
#[derive(Default, Deserialize)]
#[serde(transparent)]
pub struct ImageData(Option<String>);

impl ImageData {
    pub fn into_inner(self) -> Option<String> {
        self.0
    }

    pub fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }

    pub fn take(&mut self) -> Option<String> {
        self.0.take()
    }

    pub fn cloned(&self) -> Option<String> {
        self.0.clone()
    }
}

impl fmt::Debug for ImageData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(_) => write!(f, "Some(<image>)"),
            None => write!(f, "None"),
        }
    }
}
