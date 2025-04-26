CREATE TABLE IF NOT EXISTS user_sessions (
    id TEXT PRIMARY KEY, -- Store UUID as TEXT
    id_token TEXT NOT NULL,
    nonce TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL  -- ISO 8601 UTC
);
