SELECT
    *
FROM
    gi_weapons
WHERE
    EXISTS (
        SELECT
            *
        FROM
            gi_weapons_text
        WHERE
            gi_weapons.id = gi_weapons_text.id);

