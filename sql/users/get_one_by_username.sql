SELECT
    username,
    password,
    email
FROM
    users
WHERE
    username = $1;
