SELECT
    min(timestamp)
FROM
    warps_standard
WHERE
    uid = $1;

