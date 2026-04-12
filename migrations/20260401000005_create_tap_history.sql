CREATE TABLE IF NOT EXISTS tap_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id INTEGER NOT NULL REFERENCES pub(id),
    tap_number INTEGER NOT NULL,
    beer_id INTEGER NOT NULL REFERENCES beer(id),
    prices TEXT,
    tapped_at TEXT NOT NULL,
    removed_at TEXT NOT NULL DEFAULT (datetime('now'))
);
