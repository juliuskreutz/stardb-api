SELECT
    id
FROM
    achievements
WHERE
    id != $1
    AND SET = $2;

