CREATE TABLE bags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    roast_id INTEGER NOT NULL,
    roast_date DATE,
    amount REAL NOT NULL,
    remaining REAL NOT NULL,
    closed BOOLEAN NOT NULL DEFAULT FALSE,
    finished_at DATE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (roast_id) REFERENCES roasts (id) ON DELETE CASCADE
);

CREATE INDEX idx_bags_roast_id ON bags(roast_id);
CREATE INDEX idx_bags_closed ON bags(closed);
