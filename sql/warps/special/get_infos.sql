SELECT
    warps_special.character,
    warps_special.light_cone,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity,
    warps_special.timestamp
FROM
    warps_special
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN light_cones ON light_cones.id = light_cone
WHERE
    uid = $1
ORDER BY
    warps_special.id;

