CREATE TABLE stats_cache (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    data TEXT NOT NULL,
    computed_at TEXT NOT NULL DEFAULT (datetime('now'))
);
