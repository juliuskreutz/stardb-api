CREATE EXTENSION IF NOT EXISTS fuzzystrmatch;

CREATE TABLE IF NOT EXISTS scores (
    uid INT8 PRIMARY KEY NOT NULL,
    region TEXT NOT NULL,
    name TEXT NOt NULL,
    level INT4 NOT NULL,
    signature TEXT NOT NULL,
    avatar_icon TEXT NOT NULL,
    achievement_count INT4 NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS scores_damage (
    uid INT8 NOT NULL REFERENCES scores ON DELETE CASCADE,
    character TEXT NOT NULL,
    support BOOLEAN NOT NULL,
    damage INT4 NOT NULL,
    video TEXT NOT NULL,
    PRIMARY KEY (uid, character, support)
);

CREATE TABLE IF NOT EXISTS scores_heal (
    uid INT8 PRIMARY KEY NOT NULL REFERENCES scores ON DELETE CASCADE,
    heal INT4 NOT NULL,
    video TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scores_shield (
    uid INT8 PRIMARY KEY NOT NULL REFERENCES scores ON DELETE CASCADE,
    shield INT4 NOT NULL,
    video TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS submissions_damage (
    uuid UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    uid INT8 NOT NULL REFERENCES scores ON DELETE CASCADE,
    character TEXT NOT NULL,
    support BOOLEAN NOT NULL,
    damage INT4 NOT NULL,
    video TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE(uid, character, support)
);

CREATE TABLE IF NOT EXISTS submissions_heal (
    uuid UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    uid INT8 NOT NULL REFERENCES scores ON DELETE CASCADE,
    heal INT4 NOT NULL,
    video TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE(uid)
);

CREATE TABLE IF NOT EXISTS submissions_shield (
    uuid UUID PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    uid INT8 NOT NULL REFERENCES scores ON DELETE CASCADE,
    shield INT4 NOT NULL,
    video TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE(uid)
);

CREATE TABLE IF NOT EXISTS users (
    username TEXT PRIMARY KEY NOT NULL,
    password TEXT NOT NULL,
    email TEXT,
    admin BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS achievements (
    id INT8 PRIMARY KEY NOT NULL,
    series TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    jades INT4 NOT NULL,
    hidden BOOLEAN NOT NULL,
    comment TEXT,
    reference TEXT,
    difficulty TEXT
);

CREATE TABLE IF NOT EXISTS completed (
    username TEXT NOT NULL REFERENCES users ON DELETE CASCADE,
    id INT8 NOT NULL REFERENCES achievements ON DELETE CASCADE,
    PRIMARY KEY (username, id)
);

CREATE TABLE IF NOT EXISTS verifications (
    uid INT8 PRIMARY KEY NOT NULL REFERENCES scores ON DELETE CASCADE,
    username TEXT NOT NULL REFERENCES users ON DELETE CASCADE,
    otp TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS connections (
    uid INT8 PRIMARY KEY NOT NULL REFERENCES scores ON DELETE CASCADE,
    username TEXT NOT NULL REFERENCES users ON DELETE CASCADE
);


