UPDATE
    gi_achievements
SET
    impossible = $2
WHERE
    id = $1;

