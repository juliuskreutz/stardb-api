CREATE TABLE IF NOT EXISTS series (
    id INT4 PRIMARY KEY NOT NULL,
    tag TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    priority INT4 NOT NULL
);