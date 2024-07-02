ALTER TABLE achievements_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE achievement_series_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE books_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE book_series_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE book_series_worlds_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE characters_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE light_cones_text
    ALTER COLUMN LANGUAGE TYPE
    text;

ALTER TABLE skills_text
    ALTER COLUMN LANGUAGE TYPE
    text;

DROP TYPE LANGUAGE;

TRUNCATE achievements_text;

TRUNCATE achievement_series_text;

TRUNCATE books_text;

TRUNCATE book_series_text;

TRUNCATE book_series_worlds_text;

TRUNCATE characters_text;

TRUNCATE light_cones_text;

TRUNCATE skills_text;

