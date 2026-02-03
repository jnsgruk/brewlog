-- Add denormalized slug columns to timeline_events
-- This eliminates the need for 9-way JOINs when listing timeline events

-- Step 1: Add the new columns
ALTER TABLE timeline_events ADD COLUMN slug TEXT;
ALTER TABLE timeline_events ADD COLUMN roaster_slug TEXT;
ALTER TABLE timeline_events ADD COLUMN brew_data_json TEXT;

-- Step 2: Backfill roaster slugs for roaster events
UPDATE timeline_events
SET slug = (SELECT slug FROM roasters WHERE roasters.id = timeline_events.entity_id)
WHERE entity_type = 'roaster';

-- Step 3: Backfill slugs for roast events
UPDATE timeline_events
SET slug = (SELECT slug FROM roasts WHERE roasts.id = timeline_events.entity_id),
    roaster_slug = (
        SELECT ro.slug FROM roasts r
        JOIN roasters ro ON r.roaster_id = ro.id
        WHERE r.id = timeline_events.entity_id
    )
WHERE entity_type = 'roast';

-- Step 4: Backfill slugs for bag events
UPDATE timeline_events
SET slug = (
        SELECT r.slug FROM bags b
        JOIN roasts r ON b.roast_id = r.id
        WHERE b.id = timeline_events.entity_id
    ),
    roaster_slug = (
        SELECT ro.slug FROM bags b
        JOIN roasts r ON b.roast_id = r.id
        JOIN roasters ro ON r.roaster_id = ro.id
        WHERE b.id = timeline_events.entity_id
    )
WHERE entity_type = 'bag';

-- Step 5: Backfill slugs and brew_data for brew events
UPDATE timeline_events
SET slug = (
        SELECT r.slug FROM brews br
        JOIN bags b ON br.bag_id = b.id
        JOIN roasts r ON b.roast_id = r.id
        WHERE br.id = timeline_events.entity_id
    ),
    roaster_slug = (
        SELECT ro.slug FROM brews br
        JOIN bags b ON br.bag_id = b.id
        JOIN roasts r ON b.roast_id = r.id
        JOIN roasters ro ON r.roaster_id = ro.id
        WHERE br.id = timeline_events.entity_id
    ),
    brew_data_json = (
        SELECT json_object(
            'bag_id', br.bag_id,
            'grinder_id', br.grinder_id,
            'brewer_id', br.brewer_id,
            'coffee_weight', br.coffee_weight,
            'grind_setting', br.grind_setting,
            'water_volume', br.water_volume,
            'water_temp', br.water_temp
        )
        FROM brews br
        WHERE br.id = timeline_events.entity_id
    )
WHERE entity_type = 'brew';

-- Gear events have no slug (entity_type = 'gear' gets NULL for both slug columns)
