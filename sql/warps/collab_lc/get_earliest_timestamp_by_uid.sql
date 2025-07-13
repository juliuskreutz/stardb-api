SELECT
    min(timestamp)
FROM
    warps_collab_lc
WHERE
    uid = $1;

