SELECT
    characters.id,
    characters.rarity,
    characters_text.name,
    characters_text.path,
    characters_text.element,
    characters_text_en.path path_id,
    characters_text_en.element element_id,
    COUNT(*)
FROM (
    SELECT
        uid,
        character
    FROM
        warps_departure
    UNION ALL
    SELECT
        uid,
        character
    FROM
        warps_standard
    UNION ALL
    SELECT
        uid,
        character
    FROM
        warps_special
    UNION ALL
    SELECT
        uid,
        character
    FROM
        warps_lc) warps
    LEFT JOIN characters ON characters.id = character
    LEFT JOIN characters_text ON characters_text.id = character
        AND characters_text.language = $2
    LEFT JOIN characters_text AS characters_text_en ON characters_text_en.id = character
        AND characters_text_en.language = 'en'
WHERE
    uid = $1
    AND character IS NOT NULL
GROUP BY
    characters.id,
    characters.rarity,
    characters_text.name,
    characters_text.path,
    characters_text.element,
    characters_text_en.path,
    characters_text_en.element
ORDER BY
    rarity DESC,
    id DESC;

