ALTER TABLE achievements
    ADD COLUMN timegated boolean NOT NULL DEFAULT FALSE;

ALTER TABLE achievements
    ADD COLUMN missable boolean NOT NULL DEFAULT FALSE;

ALTER TABLE zzz_achievements
    ADD COLUMN timegated boolean NOT NULL DEFAULT FALSE;

ALTER TABLE zzz_achievements
    ADD COLUMN missable boolean NOT NULL DEFAULT FALSE;

