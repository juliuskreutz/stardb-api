ALTER TABLE series DROP COLUMN name;

CREATE TABLE IF NOT EXISTS series_text (
    id INT4 NOT NULL REFERENCES series ON DELETE CASCADE,
    language TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(id, language)
);
