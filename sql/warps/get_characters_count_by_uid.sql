-- Count numeric item identities before joining catalog/localized text. UNION ALL
-- retains every copy, including duplicates across standard and collab pools.
WITH copies AS (
    SELECT character, COUNT(*) AS count
    FROM (
        SELECT character FROM warps_departure WHERE uid = $1
        UNION ALL
        SELECT character FROM warps_standard WHERE uid = $1
        UNION ALL
        SELECT character FROM warps_special WHERE uid = $1
        UNION ALL
        SELECT character FROM warps_lc WHERE uid = $1
        UNION ALL
        SELECT character FROM warps_collab WHERE uid = $1
        UNION ALL
        SELECT character FROM warps_collab_lc WHERE uid = $1
    ) warps
    WHERE character IS NOT NULL
    GROUP BY character
)
SELECT
    characters.id,
    characters.rarity,
    characters_text.name,
    characters_text.path,
    characters_text.element,
    characters_text_en.path path_id,
    characters_text_en.element element_id,
    copies.count
FROM copies
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN characters_text ON characters_text.id = character
        AND characters_text.language = $2
    LEFT JOIN characters_text AS characters_text_en ON characters_text_en.id = character
        AND characters_text_en.language = 'en'
ORDER BY
    rarity DESC,
    id DESC;

