INSERT INTO mihomo (uid, region, name, level, signature, avatar_icon, achievement_count, updated_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (uid)
    DO UPDATE SET
        name = EXCLUDED.name, level = EXCLUDED.level, signature = EXCLUDED.signature, avatar_icon = EXCLUDED.avatar_icon, achievement_count = EXCLUDED.achievement_count, updated_at = EXCLUDED.updated_at;

