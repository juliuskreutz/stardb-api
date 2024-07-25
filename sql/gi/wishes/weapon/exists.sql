SELECT
    id,
    uid
FROM
    gi_wishes_weapon
WHERE
    id = $1
    AND uid = $2;

