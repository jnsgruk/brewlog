-- migrate:up
PRAGMA foreign_keys = ON;

CREATE TABLE roasters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    country TEXT NOT NULL,
    city TEXT,
    homepage TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE roasts (
    id TEXT PRIMARY KEY,
    roaster_id TEXT NOT NULL REFERENCES roasters(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    origin TEXT,
    region TEXT,
    producer TEXT,
    process TEXT,
    tasting_notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE cafes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    latitude REAL,
    longitude REAL,
    website TEXT,
    address TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE gear (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    gear_type TEXT NOT NULL CHECK (gear_type IN ('grinder','brewer','kettle','filter_paper')),
    manufacturer TEXT,
    model TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE brews (
    id TEXT PRIMARY KEY,
    roast_id TEXT NOT NULL REFERENCES roasts(id) ON DELETE RESTRICT,
    method TEXT NOT NULL,
    dose_grams REAL,
    water_grams REAL,
    brew_temperature_c REAL,
    grind_setting TEXT,
    brewed_at TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE brew_gear (
    brew_id TEXT NOT NULL REFERENCES brews(id) ON DELETE CASCADE,
    gear_id TEXT NOT NULL REFERENCES gear(id) ON DELETE RESTRICT,
    PRIMARY KEY (brew_id, gear_id)
);

CREATE TABLE cups (
    id TEXT PRIMARY KEY,
    cafe_id TEXT NOT NULL REFERENCES cafes(id) ON DELETE RESTRICT,
    roast_id TEXT REFERENCES roasts(id) ON DELETE SET NULL,
    price_cents INTEGER,
    consumed_at TEXT NOT NULL,
    notes TEXT,
    rating INTEGER CHECK (rating BETWEEN 0 AND 255),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX idx_roasts_roaster_id ON roasts(roaster_id);
CREATE INDEX idx_brews_roast_id ON brews(roast_id);
CREATE INDEX idx_brew_gear_gear_id ON brew_gear(gear_id);
CREATE INDEX idx_cups_cafe_id ON cups(cafe_id);
CREATE INDEX idx_cups_roast_id ON cups(roast_id);

CREATE TABLE timeline_events (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('roaster', 'roast')),
    entity_id TEXT NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    title TEXT NOT NULL,
    details_json TEXT,
    tasting_notes_json TEXT
);

CREATE INDEX idx_timeline_events_entity ON timeline_events(entity_type, entity_id);
CREATE INDEX idx_timeline_events_occurred_at ON timeline_events(occurred_at);
