CREATE TABLE IF NOT EXISTS pub_user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pub_id INTEGER NOT NULL REFERENCES pub(id),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    must_change_password BOOLEAN NOT NULL DEFAULT true,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
