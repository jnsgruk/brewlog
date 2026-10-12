-- Historical deductions cannot be inferred after clamping or manual stock edits.
-- Start legacy/imported brews without refundable stock; new deductions are tracked.
ALTER TABLE brews ADD COLUMN inventory_deducted REAL NOT NULL DEFAULT 0
    CHECK (inventory_deducted >= 0);
