SELECT
    warps_departure.id,
    warps_departure.character,
    warps_departure.light_cone,
    warps_departure.timestamp,
    COALESCE(characters_text.name, light_cones_text.name) AS name,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity
FROM
    warps_departure
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

