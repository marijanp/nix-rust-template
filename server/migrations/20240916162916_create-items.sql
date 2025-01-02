
CREATE TABLE items (
    id TEXT PRIMARY KEY NOT NULL,  -- Store UUID as TEXT in SQLite
    name TEXT NOT NULL,
    price TEXT NOT NULL  -- Store u128 as TEXT
);
