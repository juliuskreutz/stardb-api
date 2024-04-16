UPDATE
    users
SET
    email = NULL
WHERE
    username = $1;

