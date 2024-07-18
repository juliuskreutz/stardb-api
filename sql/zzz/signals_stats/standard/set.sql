INSERT INTO zzz_signals_stats_standard (uid, luck_a, luck_s)
    VALUES ($1, $2, $3)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_a = EXCLUDED.luck_a, luck_s = EXCLUDED.luck_s;

