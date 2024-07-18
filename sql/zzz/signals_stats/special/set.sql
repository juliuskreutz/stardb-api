INSERT INTO zzz_signals_stats_special (uid, luck_a, luck_s, win_rate, win_streak, loss_streak)
    VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (uid)
    DO UPDATE SET
        luck_a = EXCLUDED.luck_a, luck_s = EXCLUDED.luck_s, win_rate = EXCLUDED.win_rate, win_streak = EXCLUDED.win_streak, loss_streak = EXCLUDED.loss_streak;

