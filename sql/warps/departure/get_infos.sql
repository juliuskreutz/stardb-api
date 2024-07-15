SELECT
    warps_departure.uid,
    warps_departure.character,
    warps_departure.light_cone,
    COALESCE(characters.rarity, light_cones.rarity) AS rarity
FROM
    warps_departure
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN light_cones ON light_cones.id = light_cone
WHERE
    uid = $1
ORDER BY
    warps_departure.id;

