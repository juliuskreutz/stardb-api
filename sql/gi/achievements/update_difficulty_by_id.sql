UPDATE
    gi_achievements
SET
    difficulty = $2
WHERE
    id = $1;

