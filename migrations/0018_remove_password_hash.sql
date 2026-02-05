-- Remove password_hash column from users (passkey-only auth)
-- SQLite 3.35+ supports ALTER TABLE DROP COLUMN
ALTER TABLE users DROP COLUMN password_hash;
