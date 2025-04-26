CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY NOT NULL,  -- Store UUID as TEXT
    name TEXT NOT NULL,
    price TEXT NOT NULL  -- Store u128 as TEXT
);
