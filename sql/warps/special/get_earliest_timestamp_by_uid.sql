SELECT
    min(timestamp)
FROM
    warps_special
WHERE
    uid = $1;

