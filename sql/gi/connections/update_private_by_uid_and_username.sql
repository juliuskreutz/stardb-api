UPDATE
    gi_connections
SET
    private = $3
WHERE
    uid = $1
    AND username = $2;

