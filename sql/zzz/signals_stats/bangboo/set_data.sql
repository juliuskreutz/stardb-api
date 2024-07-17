INSERT INTO zzz_signals_stats_bangboo (uid, count_percentile, luck_a, luck_a_percentile, luck_s, luck_s_percentile)
    VALUES ($1, 0, $2, 0, $3, 0)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_a = EXCLUDED.luck_a, luck_s = EXCLUDED.luck_s;

