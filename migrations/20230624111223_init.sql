CREATE EXTENSION IF NOT EXISTS fuzzystrmatch;

CREATE TABLE IF NOT EXISTS scores (
    uid INT8 PRIMARY KEY NOT NULL,
    region TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    info JSON NOT NULL,
    updated_at TIMESTAMP DEFAULT now()
);

CREATE TABLE IF NOT EXISTS scores_damage (
    uid INT8 NOT NULL REFERENCES scores ON DELETE CASCADE,
    character TEXT NOT NULL,
    support BOOLEAN NOT NULL,
    damage INT4 NOT NULL,
    PRIMARY KEY (uid, character, support)
);

CREATE TABLE IF NOT EXISTS scores_heal (
    uid INT8 PRIMARY KEY NOT NULL REFERENCES scores ON DELETE CASCADE,
    heal INT4 NOT NULL
);

CREATE TABLE IF NOT EXISTS scores_shield (
    uid INT8 PRIMARY KEY NOT NULL REFERENCES scores ON DELETE CASCADE,
    shield INT4 NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    username TEXT PRIMARY KEY NOT NULL,
    password TEXT NOT NULL,
    email TEXT
);

CREATE TABLE IF NOT EXISTS admins (
    username TEXT PRIMARY KEY NOT NULL REFERENCES users ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS completed (
    username TEXT NOT NULL REFERENCES users ON DELETE CASCADE,
    achievement INT8 NOT NULL
);

CREATE TABLE IF NOT EXISTS achievements (
    id INT8 PRIMARY KEY NOT NULL,
    series TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    comment TEXT,
    reference TEXT,
    difficulty TEXT
);

