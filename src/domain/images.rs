/// An image associated with an entity (roaster, roast, gear, or cafe).
pub struct EntityImage {
    pub entity_type: String,
    pub entity_id: i64,
    pub content_type: String,
    pub image_data: Vec<u8>,
    pub thumbnail_data: Vec<u8>,
}
