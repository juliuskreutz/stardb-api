INSERT INTO zzz_banners (
    id, name, start, "end", character, character_gacha_type,
    w_engine, w_engine_gacha_type, bangboo, bangboo_gacha_type
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name, start = EXCLUDED.start, "end" = EXCLUDED."end",
    character = EXCLUDED.character, character_gacha_type = EXCLUDED.character_gacha_type,
    w_engine = EXCLUDED.w_engine, w_engine_gacha_type = EXCLUDED.w_engine_gacha_type,
    bangboo = EXCLUDED.bangboo, bangboo_gacha_type = EXCLUDED.bangboo_gacha_type;
