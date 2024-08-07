SELECT
    max(timestamp)
FROM
    gi_wishes_standard
WHERE
    uid = $1;

