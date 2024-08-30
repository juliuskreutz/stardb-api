DELETE FROM gi_wishes_weapon
WHERE uid = $1
    AND NOT official;

