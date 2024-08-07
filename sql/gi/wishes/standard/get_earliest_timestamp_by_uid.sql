SELECT
    min(timestamp)
FROM
    gi_wishes_standard
WHERE
    uid = $1;

