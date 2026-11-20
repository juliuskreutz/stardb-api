INSERT INTO warps_stats_standard (uid, luck_4, luck_5, pull_count)
    VALUES ($1, $2, $3, $4)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_4 = EXCLUDED.luck_4, luck_5 = EXCLUDED.luck_5, pull_count = EXCLUDED.pull_count;
