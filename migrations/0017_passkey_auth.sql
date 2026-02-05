-- Add UUID column to users for WebAuthn user handle
ALTER TABLE users ADD COLUMN uuid TEXT;

-- Backfill existing users with random v4 UUIDs
UPDATE users SET uuid =
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-' ||
    '4' || substr(lower(hex(randomblob(2))), 2) || '-' ||
    substr('89ab', abs(random()) % 4 + 1, 1) || substr(lower(hex(randomblob(2))), 2) || '-' ||
    lower(hex(randomblob(6)));

-- Passkey credential storage
CREATE TABLE passkey_credentials (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_json TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT 'default',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_used_at TEXT
);

CREATE INDEX idx_passkey_credentials_user_id ON passkey_credentials(user_id);

-- One-time registration tokens for bootstrap and invite flows
CREATE TABLE registration_tokens (
    id INTEGER PRIMARY KEY,
    token_hash TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expires_at TEXT NOT NULL,
    used_at TEXT,
    used_by_user_id INTEGER REFERENCES users(id)
);

CREATE INDEX idx_registration_tokens_token_hash ON registration_tokens(token_hash);
