DELETE FROM gi_connections
WHERE uid = $1
    AND username = $2;

