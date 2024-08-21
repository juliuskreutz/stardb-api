SELECT
    id,
    rarity
FROM
    gi_weapons
    NATURAL JOIN gi_weapons_text
WHERE
    lower(replace(replace(name, '''', ''), ' ', '_')) = $1;

