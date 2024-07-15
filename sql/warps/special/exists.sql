SELECT
    id,
    uid
FROM
    warps_special
WHERE
    id = $1
    AND uid = $2;

