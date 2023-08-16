ALTER TABLE characters DROP COLUMN name;
ALTER TABLE characters DROP COLUMN element;
ALTER TABLE characters DROP COLUMN path;

CREATE TABLE IF NOT EXISTS characters_text (
    id INT4 NOT NULL REFERENCES characters ON DELETE CASCADE,
    language TEXT NOT NULL,
    name TEXT NOT NULL,
    element TEXT NOT NULL,
    path TEXT NOT NULL,
    PRIMARY KEY(id, language)
);
