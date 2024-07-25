SELECT
    id,
    uid
FROM
    gi_wishes_beginner
WHERE
    id = $1
    AND uid = $2;

