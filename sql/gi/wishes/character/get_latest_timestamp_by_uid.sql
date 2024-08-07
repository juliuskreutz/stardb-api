SELECT
    max(timestamp)
FROM
    gi_wishes_character
WHERE
    uid = $1;

