-- Convert date-only finished_at values to full ISO 8601 datetimes.
-- Going forward, finished_at stores full datetimes for correct timeline
-- ordering (bag "finished" events must sort after same-day brews).
--
-- Strategy: set finished_at to 1 minute after the last brew from that bag.
-- If no brews exist for the bag, fall back to end-of-day (23:59:59Z).
UPDATE bags
SET finished_at = COALESCE(
    (
        SELECT strftime('%Y-%m-%dT%H:%M:%fZ', brews.created_at, '+1 minute')
        FROM brews
        WHERE brews.bag_id = bags.id
        ORDER BY brews.created_at DESC
        LIMIT 1
    ),
    finished_at || 'T23:59:59Z'
)
WHERE finished_at IS NOT NULL
  AND LENGTH(finished_at) = 10;

-- Fix existing timeline events so bag "finished" uses the actual finished_at.
UPDATE timeline_events
SET occurred_at = (
    SELECT bags.finished_at
    FROM bags
    WHERE bags.id = timeline_events.entity_id
)
WHERE entity_type = 'bag'
  AND action = 'finished'
  AND EXISTS (
    SELECT 1 FROM bags
    WHERE bags.id = timeline_events.entity_id
      AND bags.finished_at IS NOT NULL
  );
