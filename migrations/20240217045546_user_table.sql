-- Add migration script here
CREATE TABLE IF NOT EXISTS user (id INTEGER PRIMARY KEY AUTOINCREMENT,
                                 username TEXT UNIQUE NOT NULL,
                                 password_hash TEXT NOT NULL);
