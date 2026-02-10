CREATE TABLE entity_images (
    id INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('roaster', 'roast', 'gear', 'cafe', 'brew', 'cup')),
    entity_id INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    image_data BLOB NOT NULL,
    thumbnail_data BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(entity_type, entity_id)
);

CREATE INDEX idx_entity_images_lookup ON entity_images (entity_type, entity_id);
