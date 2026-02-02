-- Update timeline_events CHECK constraint to include 'gear'
-- SQLite requires recreating the table to modify CHECK constraints

-- Step 1: Rename old table
ALTER TABLE timeline_events RENAME TO timeline_events_old;

-- Step 2: Create new table with updated constraint
CREATE TABLE timeline_events (
    id INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('roaster', 'roast', 'bag', 'gear')),
    entity_id INTEGER NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    title TEXT NOT NULL,
    details_json TEXT,
    tasting_notes_json TEXT,
    action TEXT NOT NULL DEFAULT 'added'
);

-- Step 3: Copy data from old table
INSERT INTO timeline_events (id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json, action)
SELECT id, entity_type, entity_id, occurred_at, title, details_json, tasting_notes_json, action
FROM timeline_events_old;

-- Step 4: Drop old table
DROP TABLE timeline_events_old;

-- Step 5: Recreate indexes
CREATE INDEX idx_timeline_events_entity ON timeline_events(entity_type, entity_id);
CREATE INDEX idx_timeline_events_occurred_at ON timeline_events(occurred_at);
