CREATE TABLE IF NOT EXISTS scores_damage (
    uid INT8 NOT NULL REFERENCES mihomo ON DELETE CASCADE,
    character INT4 NOT NULL REFERENCES characters ON DELETE CASCADE,
    support BOOLEAN NOT NULL,
    damage INT4 NOT NULL,
    video TEXT NOT NULL,
    PRIMARY KEY (uid, character, support)
);