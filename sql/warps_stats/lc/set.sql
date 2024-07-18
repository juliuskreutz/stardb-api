INSERT INTO warps_stats_lc (uid, luck_4, luck_5, win_rate, win_streak, loss_streak)
    VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_4 = EXCLUDED.luck_4, luck_5 = EXCLUDED.luck_5, win_rate = EXCLUDED.win_rate, win_streak = EXCLUDED.win_streak, loss_streak = EXCLUDED.loss_streak;

