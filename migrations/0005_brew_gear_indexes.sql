-- Add missing indexes on brew foreign keys to gear table.
-- The brew list query joins gear 3 times (grinder, brewer, filter paper);
-- these indexes avoid full table scans on the gear side of each join.

CREATE INDEX idx_brews_grinder_id ON brews(grinder_id);
CREATE INDEX idx_brews_brewer_id ON brews(brewer_id);
CREATE INDEX idx_brews_filter_paper_id ON brews(filter_paper_id);
