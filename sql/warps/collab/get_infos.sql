SELECT
    warps_collab.character,
    warps_collab.light_cone,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity,
    warps_collab.timestamp
FROM
    warps_collab
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN light_cones ON light_cones.id = light_cone
WHERE
    uid = $1
ORDER BY
    warps_collab.id;

