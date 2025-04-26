CREATE TABLE IF NOT EXISTS authentication_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    csrf_token TEXT NOT NULL UNIQUE,
    nonce TEXT NOT NULL UNIQUE,
    created_at TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')) -- ISO 8601 UTC
);
