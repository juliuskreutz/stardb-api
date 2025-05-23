SELECT
    warps_standard.character,
    warps_standard.light_cone,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity,
    warps_standard.timestamp
FROM
    warps_standard
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN light_cones ON light_cones.id = light_cone
WHERE
    uid = $1
ORDER BY
    warps_standard.id;

