-- Brew, cafe, and cup tables

CREATE TABLE brews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bag_id INTEGER NOT NULL REFERENCES bags(id) ON DELETE CASCADE,
    coffee_weight REAL NOT NULL,
    grinder_id INTEGER NOT NULL REFERENCES gear(id) ON DELETE RESTRICT,
    grind_setting REAL NOT NULL,
    brewer_id INTEGER NOT NULL REFERENCES gear(id) ON DELETE RESTRICT,
    filter_paper_id INTEGER REFERENCES gear(id) ON DELETE SET NULL,
    water_volume INTEGER NOT NULL,
    water_temp REAL NOT NULL,
    quick_notes TEXT,
    brew_time INTEGER,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_brews_bag_id ON brews(bag_id);
CREATE INDEX idx_brews_created_at ON brews(created_at DESC);

CREATE TABLE cafes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    city TEXT NOT NULL,
    country TEXT NOT NULL,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    website TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE UNIQUE INDEX idx_cafes_slug ON cafes(slug);

CREATE TABLE cups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    roast_id INTEGER NOT NULL REFERENCES roasts(id) ON DELETE RESTRICT,
    cafe_id INTEGER NOT NULL REFERENCES cafes(id) ON DELETE RESTRICT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX idx_cups_roast_id ON cups(roast_id);
CREATE INDEX idx_cups_cafe_id ON cups(cafe_id);
