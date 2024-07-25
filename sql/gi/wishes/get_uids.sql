SELECT
    uid
FROM
    gi_profiles
WHERE
    EXISTS (
        SELECT
            *
        FROM (
            SELECT
                uid
            FROM
                gi_wishes_beginner
            UNION ALL
            SELECT
                uid
            FROM
                gi_wishes_standard
            UNION ALL
            SELECT
                uid
            FROM
                gi_wishes_character
            UNION ALL
            SELECT
                uid
            FROM
                gi_wishes_weapon
            UNION ALL
            SELECT
                uid
            FROM
                gi_wishes_chronicled) gi_wishes
        WHERE
            gi_profiles.uid = gi_wishes.uid)
    AND NOT EXISTS (
        SELECT
            *
        FROM
            gi_connections
        WHERE
            gi_profiles.uid = gi_connections.uid
            AND gi_connections.private);

