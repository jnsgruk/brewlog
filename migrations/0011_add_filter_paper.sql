-- Add 'filter_paper' gear category and optional filter_paper_id on brews.
--
-- SQLite does not support altering CHECK constraints, so we recreate the gear
-- table with the updated constraint.

CREATE TABLE gear_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL CHECK (category IN ('grinder', 'brewer', 'filter_paper')),
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO gear_new (id, category, make, model, created_at, updated_at)
SELECT id, category, make, model, created_at, updated_at FROM gear;

DROP TABLE gear;
ALTER TABLE gear_new RENAME TO gear;
CREATE INDEX idx_gear_category ON gear(category);

ALTER TABLE brews ADD COLUMN filter_paper_id INTEGER REFERENCES gear(id) ON DELETE SET NULL;
