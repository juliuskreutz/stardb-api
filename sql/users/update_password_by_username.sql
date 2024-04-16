UPDATE
    users
SET
    PASSWORD = $2
WHERE
    username = $1;

