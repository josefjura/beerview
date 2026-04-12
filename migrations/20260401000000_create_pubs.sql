CREATE TABLE IF NOT EXISTS pub (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    neighbourhood TEXT,
    tap_count INTEGER NOT NULL DEFAULT 4,
    webhook_url TEXT,
    is_offline BOOLEAN NOT NULL DEFAULT false,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
