CREATE TABLE IF NOT EXISTS mihomo (
    uid INT8 PRIMARY KEY NOT NULL,
    region TEXT NOT NULL,
    name TEXT NOt NULL,
    level INT4 NOT NULL,
    signature TEXT NOT NULL,
    avatar_icon TEXT NOT NULL,
    achievement_count INT4 NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);