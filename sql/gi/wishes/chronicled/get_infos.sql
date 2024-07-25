SELECT
    gi_wishes_chronicled.uid,
    gi_wishes_chronicled.character,
    gi_wishes_chronicled.weapon,
    COALESCE(gi_characters.rarity, gi_weapons.rarity) AS rarity
FROM
    gi_wishes_chronicled
    LEFT JOIN gi_characters ON gi_characters.id = character
    LEFT JOIN gi_weapons ON gi_weapons.id = weapon
WHERE
    uid = $1
ORDER BY
    gi_wishes_chronicled.id;

