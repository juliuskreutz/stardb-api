SELECT
    light_cones.id,
    light_cones.rarity,
    light_cones_text.name,
    light_cones_text.path,
    light_cones_text_en.path AS path_id,
    COUNT(*)
FROM (
    SELECT
        uid,
        light_cone
    FROM
        warps_departure
    UNION ALL
    SELECT
        uid,
        light_cone
    FROM
        warps_standard
    UNION ALL
    SELECT
        uid,
        light_cone
    FROM
        warps_special
    UNION ALL
    SELECT
        uid,
        light_cone
    FROM
        warps_lc
    UNION ALL
    SELECT
        uid,
        light_cone
    FROM
        warps_collab
    UNION ALL
    SELECT
        uid,
        light_cone
    FROM
        warps_collab_lc) warps
    LEFT JOIN light_cones ON light_cones.id = light_cone
    LEFT JOIN light_cones_text ON light_cones_text.id = light_cone
        AND light_cones_text.language = $2
    LEFT JOIN light_cones_text AS light_cones_text_en ON light_cones_text_en.id = light_cone
        AND light_cones_text_en.language = 'en'
WHERE
    uid = $1
    AND light_cone IS NOT NULL
GROUP BY
    light_cones.id,
    light_cones.rarity,
    light_cones_text.name,
    light_cones_text.path,
    light_cones_text_en.path
ORDER BY
    rarity DESC,
    id DESC;

