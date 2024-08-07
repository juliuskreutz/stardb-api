SELECT
    max(timestamp)
FROM
    gi_wishes_weapon
WHERE
    uid = $1;

