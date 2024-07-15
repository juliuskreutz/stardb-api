INSERT INTO warps_stats_standard (uid, count_percentile, luck_4, luck_4_percentile, luck_5, luck_5_percentile)
    VALUES ($1, 0, $2, 0, $3, 0)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_4 = EXCLUDED.luck_4, luck_5 = EXCLUDED.luck_5;

