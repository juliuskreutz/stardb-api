SELECT
    count(*)
FROM
    gi_wishes_character
WHERE
    uid = $1;

