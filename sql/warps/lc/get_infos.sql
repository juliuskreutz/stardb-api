SELECT
    warps_lc.character,
    warps_lc.light_cone,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity,
    warps_lc.timestamp
FROM
    warps_lc
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN light_cones ON light_cones.id = light_cone
WHERE
    uid = $1
ORDER BY
    warps_lc.id;

