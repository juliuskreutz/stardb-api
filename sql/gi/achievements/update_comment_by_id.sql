UPDATE
    gi_achievements
SET
    comment = $2
WHERE
    id = $1;

