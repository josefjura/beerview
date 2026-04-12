CREATE TABLE IF NOT EXISTS queue_item (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id INTEGER NOT NULL REFERENCES pub(id),
    beer_id INTEGER NOT NULL REFERENCES beer(id),
    prices TEXT,
    position INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
