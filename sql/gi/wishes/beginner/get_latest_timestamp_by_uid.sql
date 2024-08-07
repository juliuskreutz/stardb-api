SELECT
    max(timestamp)
FROM
    gi_wishes_beginner
WHERE
    uid = $1;

