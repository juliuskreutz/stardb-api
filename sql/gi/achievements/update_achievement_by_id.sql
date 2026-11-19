UPDATE
    gi_achievements
SET
    version = COALESCE($2, version),
    comment = COALESCE($3, comment),
    reference = COALESCE($4, reference),
    difficulty = COALESCE($5, difficulty),
    video = COALESCE($6, video),
    gacha = COALESCE($7, gacha),
    timegated = COALESCE($8, timegated),
    missable = COALESCE($9, missable),
    impossible = COALESCE($10, impossible),
    "set" = COALESCE($11, "set")
WHERE
    id = COALESCE($1, id);

