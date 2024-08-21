SELECT
    max(timestamp)
FROM
    warps_departure
WHERE
    uid = $1;

