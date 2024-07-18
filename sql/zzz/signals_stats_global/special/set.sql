INSERT INTO zzz_signals_stats_global_special (uid, count_percentile, luck_a_percentile, luck_s_percentile)
    VALUES ($1, $2, $3, $4)
ON CONFLICT (uid)
    DO UPDATE SET
        count_percentile = EXCLUDED.count_percentile, luck_a_percentile = EXCLUDED.luck_a_percentile, luck_s_percentile = EXCLUDED.luck_s_percentile;

