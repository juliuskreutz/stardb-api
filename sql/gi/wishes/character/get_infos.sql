SELECT
    gi_wishes_character.character,
    gi_wishes_character.weapon,
    COALESCE(gi_characters.rarity, gi_weapons.rarity) AS rarity,
    gi_wishes_character.timestamp
FROM
    gi_wishes_character
    LEFT JOIN gi_characters ON gi_characters.id = character
    LEFT JOIN gi_weapons ON gi_weapons.id = weapon
WHERE
    uid = $1
ORDER BY
    gi_wishes_character.id;

