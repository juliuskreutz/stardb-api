DELETE FROM gi_wishes_standard
WHERE uid = $1
    AND NOT official;

