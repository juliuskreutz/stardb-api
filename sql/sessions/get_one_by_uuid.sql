SELECT
    *
FROM
    sessions
WHERE
    uuid = $1
    AND expiry > NOW();

