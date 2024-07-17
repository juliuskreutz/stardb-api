INSERT INTO zzz_signals_stats_special (uid, count_percentile, luck_a, luck_a_percentile, luck_s, luck_s_percentile, win_rate, win_streak, loss_streak)
    VALUES ($1, 0, $2, 0, $3, 0, $4, $5, $6)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_a = EXCLUDED.luck_a, luck_s = EXCLUDED.luck_s, win_rate = EXCLUDED.win_rate, win_streak = EXCLUDED.win_streak, loss_streak = EXCLUDED.loss_streak;

