CREATE TABLE IF NOT EXISTS achievements (
    id INT8 PRIMARY KEY NOT NULL,
    series TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    comment TEXT,
    reference TEXT,
    difficulty TEXT
);
