CREATE TABLE IF NOT EXISTS sessions_new (
    uuid UUID NOT NULL PRIMARY KEY,
    username TEXT NOT NULL REFERENCES users ON DELETE CASCADE,
    expiry TIMESTAMP NOT NULL
);

INSERT INTO
    sessions_new(uuid, username, expiry)
SELECT
    uuid, SUBSTRING(value->>'username', 2, LENGTH(value->>'username') - 2) username, expiry
FROM
    sessions;

DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;
