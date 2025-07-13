SELECT
    max(timestamp)
FROM
    warps_collab
WHERE
    uid = $1;

