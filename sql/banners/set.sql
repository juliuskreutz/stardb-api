INSERT INTO banners (id, name, start, "end", character, light_cone)
    VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id)
    DO UPDATE SET
        name = excluded.name,
        start = excluded.start,
        "end" = excluded."end",
        character = excluded.character,
        light_cone = excluded.light_cone;
