CREATE TABLE IF NOT EXISTS users (
    username TEXT PRIMARY KEY NOT NULL,
    password TEXT NOT NULL,
    email TEXT,
    admin BOOLEAN NOT NULL DEFAULT false
);