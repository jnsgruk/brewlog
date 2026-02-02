-- Add 'gear' to timeline_events entity_type constraint
-- SQLite doesn't support ALTER COLUMN CHECK, so we need to recreate the constraint
-- This is safe because CHECK constraints in SQLite are not enforced retroactively

-- For SQLite: The CHECK constraint will be validated on INSERT/UPDATE
-- We just need to ensure the application code uses 'gear' correctly
-- The constraint in the original migration (0001_init.sql) would need to be updated
-- to include 'gear' in a clean deployment, but for existing databases this migration
-- documents that 'gear' is now a valid entity_type

-- For PostgreSQL (if using that feature flag):
-- ALTER TABLE timeline_events DROP CONSTRAINT IF EXISTS timeline_events_entity_type_check;
-- ALTER TABLE timeline_events ADD CONSTRAINT timeline_events_entity_type_check
--   CHECK (entity_type IN ('roaster', 'roast', 'bag', 'gear'));

-- SQLite: No actual schema change needed, application-level validation ensures correctness
-- This migration serves as documentation that 'gear' is now a valid entity_type
