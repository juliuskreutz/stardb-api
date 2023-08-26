CREATE TABLE IF NOT EXISTS books (
    id INT8 PRIMARY KEY NOT NULL,
    series INT4 NOT NULL REFERENCES book_series,
    series_inside INT4 NOT NULL
);