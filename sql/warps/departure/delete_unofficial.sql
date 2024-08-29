DELETE FROM warps_departure
WHERE uid = $1
    AND NOT official;

