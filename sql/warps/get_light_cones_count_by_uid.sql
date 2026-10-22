-- Count numeric item identities before joining catalog/localized text. UNION ALL
-- retains every copy, including duplicates across standard and collab pools.
WITH copies AS (
    SELECT light_cone, COUNT(*) AS count
    FROM (
        SELECT light_cone FROM warps_departure WHERE uid = $1
        UNION ALL
        SELECT light_cone FROM warps_standard WHERE uid = $1
        UNION ALL
        SELECT light_cone FROM warps_special WHERE uid = $1
        UNION ALL
        SELECT light_cone FROM warps_lc WHERE uid = $1
        UNION ALL
        SELECT light_cone FROM warps_collab WHERE uid = $1
        UNION ALL
        SELECT light_cone FROM warps_collab_lc WHERE uid = $1
    ) warps
    WHERE light_cone IS NOT NULL
    GROUP BY light_cone
)
SELECT
    light_cones.id,
    light_cones.rarity,
    light_cones_text.name,
    light_cones_text.path,
    light_cones_text_en.path AS path_id,
    copies.count
FROM copies
    LEFT JOIN light_cones ON light_cones.id = light_cone
    LEFT JOIN light_cones_text ON light_cones_text.id = light_cone
        AND light_cones_text.language = $2
    LEFT JOIN light_cones_text AS light_cones_text_en ON light_cones_text_en.id = light_cone
        AND light_cones_text_en.language = 'en'
ORDER BY
    rarity DESC,
    id DESC;

