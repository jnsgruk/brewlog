PRAGMA foreign_keys = ON;

CREATE TABLE roasters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    country TEXT NOT NULL,
    city TEXT,
    homepage TEXT,
    notes TEXT,
    slug TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX idx_roasters_slug ON roasters(slug);

CREATE TABLE roasts (
    id INTEGER PRIMARY KEY,
    roaster_id INTEGER NOT NULL REFERENCES roasters(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    origin TEXT,
    region TEXT,
    producer TEXT,
    process TEXT,
    tasting_notes TEXT,
    slug TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_roasts_roaster_id ON roasts(roaster_id);
CREATE UNIQUE INDEX idx_roasts_roaster_slug ON roasts(roaster_id, slug);

CREATE TABLE timeline_events (
    id INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('roaster', 'roast', 'bag')),
    entity_id INTEGER NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    title TEXT NOT NULL,
    details_json TEXT,
    tasting_notes_json TEXT
);

CREATE INDEX idx_timeline_events_entity ON timeline_events(entity_type, entity_id);
CREATE INDEX idx_timeline_events_occurred_at ON timeline_events(occurred_at);
