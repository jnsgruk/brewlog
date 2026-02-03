-- Add cups table for tracking roasts consumed at cafes
CREATE TABLE cups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    roast_id INTEGER NOT NULL REFERENCES roasts(id) ON DELETE RESTRICT,
    cafe_id INTEGER NOT NULL REFERENCES cafes(id) ON DELETE RESTRICT,
    notes TEXT,
    rating INTEGER CHECK (rating BETWEEN 1 AND 5),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_cups_roast_id ON cups(roast_id);
CREATE INDEX idx_cups_cafe_id ON cups(cafe_id);

-- Update timeline_events CHECK constraint to include 'cup'
-- SQLite requires recreating the table to modify CHECK constraints
DROP TABLE IF EXISTS timeline_events_new;
CREATE TABLE timeline_events_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('roaster', 'roast', 'bag', 'gear', 'brew', 'cafe', 'cup')),
    entity_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    title TEXT NOT NULL,
    details_json TEXT,
    tasting_notes_json TEXT,
    slug TEXT,
    roaster_slug TEXT,
    brew_data_json TEXT
);

INSERT INTO timeline_events_new (id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json)
SELECT id, entity_type, entity_id, action, occurred_at, title, details_json, tasting_notes_json, slug, roaster_slug, brew_data_json
FROM timeline_events;

DROP TABLE timeline_events;
ALTER TABLE timeline_events_new RENAME TO timeline_events;
CREATE INDEX idx_timeline_events_entity ON timeline_events(entity_type, entity_id);
CREATE INDEX idx_timeline_events_occurred_at ON timeline_events(occurred_at DESC);
