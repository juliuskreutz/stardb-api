SELECT
    username,
    password,
    email
FROM
    users
WHERE
    LOWER(email) = LOWER($1);
