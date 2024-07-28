UPDATE
    gi_achievements
SET
    version = NULL
WHERE
    id = $1;

