ALTER TABLE gi_achievements
    DROP COLUMN timegated;

ALTER TABLE gi_achievements
    ADD COLUMN timegated text;

