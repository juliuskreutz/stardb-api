SELECT
    gi_wishes_character.id,
    gi_wishes_character.uid,
    gi_wishes_character.character,
    gi_wishes_character.weapon,
    gi_wishes_character.timestamp,
    gi_wishes_character.official,
    COALESCE(gi_characters_text.name, gi_weapons_text.name) AS name,
    COALESCE(gi_characters.rarity, gi_weapons.rarity) AS rarity
FROM
    gi_wishes_character
    LEFT JOIN gi_characters ON gi_characters.id = character
    LEFT JOIN gi_weapons ON gi_weapons.id = weapon
    LEFT JOIN gi_characters_text ON gi_characters_text.id = character
        AND gi_characters_text.language = $2
    LEFT JOIN gi_weapons_text ON gi_weapons_text.id = weapon
        AND gi_weapons_text.language = $2
WHERE
    uid = $1
ORDER BY
    id;

