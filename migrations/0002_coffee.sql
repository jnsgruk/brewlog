-- Coffee supply chain and equipment tables

CREATE TABLE roasters (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    country TEXT NOT NULL,
    city TEXT,
    homepage TEXT,
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

CREATE TABLE bags (
    id INTEGER PRIMARY KEY,
    roast_id INTEGER NOT NULL REFERENCES roasts(id) ON DELETE CASCADE,
    roast_date TEXT,
    amount REAL NOT NULL,
    remaining REAL NOT NULL,
    closed BOOLEAN NOT NULL DEFAULT FALSE,
    finished_at TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_bags_roast_id ON bags(roast_id);
CREATE INDEX idx_bags_closed ON bags(closed);

CREATE TABLE gear (
    id INTEGER PRIMARY KEY,
    category TEXT NOT NULL CHECK (category IN ('grinder', 'brewer', 'filter_paper')),
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_gear_category ON gear(category);
