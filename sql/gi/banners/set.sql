INSERT INTO gi_banners (id, name, start, "end", character, character_gacha_type, weapon, weapon_gacha_type)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (id)
    DO UPDATE SET
        name = excluded.name,
        start = excluded.start,
        "end" = excluded."end",
        character = excluded.character,
        character_gacha_type = excluded.character_gacha_type,
        weapon = excluded.weapon,
        weapon_gacha_type = excluded.weapon_gacha_type;
