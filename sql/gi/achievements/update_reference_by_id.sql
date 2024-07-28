UPDATE
    gi_achievements
SET
    reference = $2
WHERE
    id = $1;

