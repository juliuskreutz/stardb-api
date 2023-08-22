ALTER TABLE achievements DROP COLUMN name;
ALTER TABLE achievements DROP COLUMN description;

CREATE TABLE IF NOT EXISTS achievements_text (
    id INT8 NOT NULL REFERENCES achievements ON DELETE CASCADE,
    language TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    PRIMARY KEY(id, language)
);
