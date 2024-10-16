ALTER TABLE achievements
    ALTER COLUMN timegated DROP NOT NULL;

ALTER TABLE zzz_achievements
    ALTER COLUMN timegated DROP NOT NULL;

