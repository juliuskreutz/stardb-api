INSERT INTO warps_stats_lc (uid, count_percentile, luck_4, luck_4_percentile, luck_5, luck_5_percentile, win_rate, win_streak, loss_streak)
    VALUES ($1, 0, $2, 0, $3, 0, $4, $5, $6)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_4 = EXCLUDED.luck_4, luck_5 = EXCLUDED.luck_5, win_rate = EXCLUDED.win_rate, win_streak = EXCLUDED.win_streak, loss_streak = EXCLUDED.loss_streak;

