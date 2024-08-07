SELECT
    min(timestamp)
FROM
    gi_wishes_beginner
WHERE
    uid = $1;

