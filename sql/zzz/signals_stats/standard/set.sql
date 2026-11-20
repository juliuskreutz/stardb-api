INSERT INTO zzz_signals_stats_standard (uid, luck_a, luck_s, pull_count)
    VALUES ($1, $2, $3, $4)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_a = EXCLUDED.luck_a, luck_s = EXCLUDED.luck_s, pull_count = EXCLUDED.pull_count;
