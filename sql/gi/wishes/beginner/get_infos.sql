SELECT
    gi_wishes_beginner.uid,
    gi_wishes_beginner.character,
    gi_wishes_beginner.weapon,
    COALESCE(gi_characters.rarity, gi_weapons.rarity) AS rarity
FROM
    gi_wishes_beginner
    LEFT JOIN gi_characters ON gi_characters.id = character
    LEFT JOIN gi_weapons ON gi_weapons.id = weapon
WHERE
    uid = $1
ORDER BY
    gi_wishes_beginner.id;

