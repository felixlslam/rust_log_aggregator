-- Add migration script here
CREATE TABLE IF NOT EXISTS logevents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            app TEXT NOT NULL,
            host TEXT NOT NULL,
            filename TEXT NOT NULL,
            log TEXT NOT NULL
);