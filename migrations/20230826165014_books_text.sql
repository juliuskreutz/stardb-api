CREATE TABLE IF NOT EXISTS books_text (
    id INT8 NOT NULL REFERENCES books ON DELETE CASCADE,
    language TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(id, language)
);