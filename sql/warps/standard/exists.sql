SELECT
    id,
    uid
FROM
    warps_standard
WHERE
    id = $1
    AND uid = $2;

