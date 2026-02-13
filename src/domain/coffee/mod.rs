pub mod bags;
pub mod brews;
pub mod cafes;
pub mod cups;
pub mod gear;
pub mod nearby_cafes;
pub mod roasters;
pub mod roasts;

/// Trims an optional string field, converting empty/whitespace-only values to `None`.
pub(crate) fn normalize_optional_field(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
