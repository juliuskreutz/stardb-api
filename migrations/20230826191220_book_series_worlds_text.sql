CREATE TABLE IF NOT EXISTS book_series_worlds_text (
    id INT4 NOT NULL REFERENCES book_series_worlds ON DELETE CASCADE,
    language TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(id, language)
);