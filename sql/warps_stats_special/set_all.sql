INSERT INTO warps_stats_special (uid, count, count_rank, luck_4, luck_4_rank, luck_5, luck_5_rank, win_rate, win_streak, loss_streak)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::integer[], $3::integer[], $4::double precision[], $5::integer[], $6::double precision[], $7::integer[], $8::double precision[], $9::integer[], $10::integer[])
ON CONFLICT (uid)
    DO UPDATE SET
        count = EXCLUDED.count,
        count_rank = EXCLUDED.count_rank,
        luck_4 = EXCLUDED.luck_4,
        luck_4_rank = EXCLUDED.luck_4_rank,
        luck_5 = EXCLUDED.luck_5,
        luck_5_rank = EXCLUDED.luck_5_rank,
        win_rate = EXCLUDED.win_rate,
        win_streak = EXCLUDED.win_streak,
        loss_streak = EXCLUDED.loss_streak;

