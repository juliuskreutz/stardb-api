SELECT
    id,
    uid
FROM
    warps_departure
WHERE
    id = $1
    AND uid = $2;

