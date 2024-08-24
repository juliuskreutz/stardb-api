SELECT
    warps_standard.id,
    warps_standard.character,
    warps_standard.light_cone,
    warps_standard.timestamp,
    warps_standard.official,
    COALESCE(characters_text.name, light_cones_text.name) AS name,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity
FROM
    warps_standard
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN light_cones ON light_cones.id = light_cone
    LEFT JOIN characters_text ON characters_text.id = character
        AND characters_text.language = $2
    LEFT JOIN light_cones_text ON light_cones_text.id = light_cone
        AND light_cones_text.language = $2
WHERE
    uid = $1
ORDER BY
    id;

