CREATE TABLE IF NOT EXISTS completed (
    username TEXT NOT NULL REFERENCES users ON DELETE CASCADE,
    id INT8 NOT NULL REFERENCES achievements ON DELETE CASCADE,
    PRIMARY KEY (username, id)
);