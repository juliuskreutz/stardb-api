UPDATE
    users
SET
    email = LOWER($2)
WHERE
    username = $1;

