SELECT
    id,
    uid
FROM
    warps_lc
WHERE
    id = $1
    AND uid = $2;

