CREATE TABLE IF NOT EXISTS achievements (
    id INT8 PRIMARY KEY NOT NULL,
    series INT4 NOT NULL REFERENCES series ON DELETE CASCADE,
    tag TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    jades INT4 NOT NULL,
    hidden BOOLEAN NOT NULL,
    version TEXT,
    comment TEXT,
    reference TEXT,
    difficulty TEXT,
    gacha BOOLEAN NOT NULL DEFAULT false,
    set INT4,
    priority INT4 NOT NULL
);