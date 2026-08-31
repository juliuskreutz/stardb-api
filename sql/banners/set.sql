INSERT INTO banners (id, name, start, "end", character, character_gacha_type, light_cone, light_cone_gacha_type)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (id)
    DO UPDATE SET
        name = excluded.name,
        start = excluded.start,
        "end" = excluded."end",
        character = excluded.character,
        character_gacha_type = excluded.character_gacha_type,
        light_cone = excluded.light_cone,
        light_cone_gacha_type = excluded.light_cone_gacha_type;
