CREATE TABLE IF NOT EXISTS tap (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id INTEGER NOT NULL REFERENCES pub(id),
    tap_number INTEGER NOT NULL,
    beer_id INTEGER REFERENCES beer(id),
    prices TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(pub_id, tap_number)
);
