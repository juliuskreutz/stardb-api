CREATE TABLE IF NOT EXISTS community_tier_list_entries (
    character INT4 NOT NULL REFERENCES characters ON DELETE CASCADE,
    eidolon INT4 NOT NULL,
    average FLOAT8 NOT NULL,
    variance FLOAT8 NOT NULL,
    votes INT4 NOT NULL,
    PRIMARY KEY(character, eidolon)
);