SELECT
    id
FROM
    zzz_achievements
WHERE
    id != $1
    AND SET = $2;

