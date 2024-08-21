SELECT
    max(timestamp)
FROM
    warps_lc
WHERE
    uid = $1;

