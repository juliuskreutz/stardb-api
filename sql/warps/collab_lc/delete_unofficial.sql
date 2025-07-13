DELETE FROM warps_collab_lc
WHERE uid = $1
    AND NOT official;

