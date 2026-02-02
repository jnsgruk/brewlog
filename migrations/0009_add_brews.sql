-- Brews table for logging individual coffee brews
CREATE TABLE brews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bag_id INTEGER NOT NULL REFERENCES bags(id) ON DELETE CASCADE,
    coffee_weight REAL NOT NULL,
    grinder_id INTEGER NOT NULL REFERENCES gear(id) ON DELETE RESTRICT,
    grind_setting REAL NOT NULL,
    brewer_id INTEGER NOT NULL REFERENCES gear(id) ON DELETE RESTRICT,
    water_volume INTEGER NOT NULL,
    water_temp REAL NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_brews_bag_id ON brews(bag_id);
CREATE INDEX idx_brews_created_at ON brews(created_at DESC);

-- Update timeline constraint to include brew entity type
DROP TABLE IF EXISTS timeline_events_new;
CREATE TABLE timeline_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('roaster', 'roast', 'bag', 'gear', 'brew')),
    entity_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    title TEXT NOT NULL,
    details_json TEXT,
    tasting_notes_json TEXT
);

INSERT INTO timeline_events_new (id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json)
SELECT id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json
FROM timeline_events;

DROP TABLE timeline_events;
ALTER TABLE timeline_events_new RENAME TO timeline_events;
CREATE INDEX idx_timeline_events_entity ON timeline_events(entity_type, entity_id);
CREATE INDEX idx_timeline_events_occurred_at ON timeline_events(occurred_at DESC);
