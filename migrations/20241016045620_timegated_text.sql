ALTER TABLE achievements
    ALTER COLUMN timegated TYPE text;

ALTER TABLE achievements
    ALTER COLUMN timegated SET DEFAULT NULL;

ALTER TABLE zzz_achievements
    ALTER COLUMN timegated TYPE text;

ALTER TABLE zzz_achievements
    ALTER COLUMN timegated SET DEFAULT NULL;

