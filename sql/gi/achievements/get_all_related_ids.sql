SELECT
    id
FROM
    gi_achievements
WHERE
    id != $1
    AND SET = $2;

