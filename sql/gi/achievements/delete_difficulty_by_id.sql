UPDATE
    gi_achievements
SET
    difficulty = NULL
WHERE
    id = $1;

