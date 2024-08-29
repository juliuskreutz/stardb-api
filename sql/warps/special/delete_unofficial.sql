DELETE FROM warps_special
WHERE uid = $1
    AND NOT official;

