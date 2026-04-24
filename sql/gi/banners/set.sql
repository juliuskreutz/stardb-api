INSERT INTO gi_banners (id, name, start, "end", character, weapon)
    VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id)
    DO UPDATE SET
        name = excluded.name,
        start = excluded.start,
        "end" = excluded."end",
        character = excluded.character,
        weapon = excluded.weapon;
