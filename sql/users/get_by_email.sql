SELECT
    username,
    password,
    email
FROM
    users
WHERE
    email = $1;
