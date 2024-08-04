INSERT INTO gi_wishes_stats_chronicled (uid, luck_4, luck_5)
    VALUES ($1, $2, $3)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_4 = EXCLUDED.luck_4, luck_5 = EXCLUDED.luck_5;

