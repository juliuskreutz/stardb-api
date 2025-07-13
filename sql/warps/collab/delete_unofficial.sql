DELETE FROM warps_collab
WHERE uid = $1
    AND NOT official;

