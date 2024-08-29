DELETE FROM warps_lc
WHERE uid = $1
    AND NOT official;

