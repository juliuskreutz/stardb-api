SELECT
    id,
    uid
FROM
    gi_wishes_chronicled
WHERE
    id = $1
    AND uid = $2;

