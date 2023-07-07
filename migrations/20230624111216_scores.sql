CREATE TABLE IF NOT EXISTS scores (
    uid INT8 PRIMARY KEY NOT NULL,
    region TEXT NOT NULL,
    name TEXT NOT NULL,
    level INT4 NOT NULL,
    avatar_icon TEXT NOT NULL,
    signature TEXT NOT NULL,
    character_count INT4 NOT NULL,
    achievement_count INT4 NOT NULL,
    character_name TEXT NOT NULL,
    character_icon TEXT NOT NULL,
    path_icon TEXT NOT NULL,
    element_color TEXT NOT NULL,
    element_icon TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL
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

