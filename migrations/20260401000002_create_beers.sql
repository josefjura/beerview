CREATE TABLE IF NOT EXISTS beer (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    brewery TEXT NOT NULL,
    style TEXT,
    abv REAL,
    untappd_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
