SELECT
    count(*)
FROM
    warps_departure
WHERE
    uid = $1;

