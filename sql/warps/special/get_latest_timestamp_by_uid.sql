SELECT
    max(timestamp)
FROM
    warps_special
WHERE
    uid = $1;

