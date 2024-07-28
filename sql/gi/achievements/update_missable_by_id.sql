UPDATE
    gi_achievements
SET
    missable = $2
WHERE
    id = $1;

