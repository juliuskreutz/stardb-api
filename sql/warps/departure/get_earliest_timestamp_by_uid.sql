SELECT
    min(timestamp)
FROM
    warps_departure
WHERE
    uid = $1;

