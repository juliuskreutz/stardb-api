UPDATE
    gi_achievements
SET
    version = $2,
    comment = $3,
    reference = $4,
    difficulty = $5,
    video = $6,
    gacha = $7,
    timegated = $8,
    missable = $9,
    impossible = $10,
    "set" = $11
WHERE
    id = $1;

