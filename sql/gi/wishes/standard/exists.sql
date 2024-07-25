SELECT
    id,
    uid
FROM
    gi_wishes_standard
WHERE
    id = $1
    AND uid = $2;

