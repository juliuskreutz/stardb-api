UPDATE
    sessions
SET
    expiry = $2
WHERE
    uuid = $1;

