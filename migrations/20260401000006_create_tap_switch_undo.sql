CREATE TABLE IF NOT EXISTS tap_switch_undo (
    pub_id INTEGER NOT NULL,
    tap_number INTEGER NOT NULL,
    prev_beer_id INTEGER,
    prev_prices TEXT,
    switched_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (pub_id, tap_number)
);
