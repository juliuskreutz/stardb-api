SELECT
    id,
    uid
FROM
    gi_wishes_character
WHERE
    id = $1
    AND uid = $2;

