SELECT
    id,
    rarity
FROM
    gi_characters
    NATURAL JOIN gi_characters_text
WHERE
    lower(replace(replace(name, '''', ''), ' ', '_')) = $1;

