DELETE FROM warps_standard
WHERE uid = $1
    AND NOT official;

