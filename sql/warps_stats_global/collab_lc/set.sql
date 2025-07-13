INSERT INTO warps_stats_global_collab_lc (uid, count_percentile, luck_4_percentile, luck_5_percentile)
    VALUES ($1, $2, $3, $4)
ON CONFLICT (uid)
    DO UPDATE SET
        count_percentile = EXCLUDED.count_percentile, luck_4_percentile = EXCLUDED.luck_4_percentile, luck_5_percentile = EXCLUDED.luck_5_percentile;

