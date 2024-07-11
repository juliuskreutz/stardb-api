INSERT INTO warps_stats_special (uid, count_percentile, luck_4, luck_4_percentile, luck_5, luck_5_percentile, win_rate, win_streak, loss_streak)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::double precision[], $3::double precision[], $4::double precision[], $5::double precision[], $6::double precision[], $7::double precision[], $8::integer[], $9::integer[])
ON CONFLICT (uid)
    DO UPDATE SET
        count_percentile = EXCLUDED.count_percentile,
        luck_4 = EXCLUDED.luck_4,
        luck_4_percentile = EXCLUDED.luck_4_percentile,
        luck_5 = EXCLUDED.luck_5,
        luck_5_percentile = EXCLUDED.luck_5_percentile,
        win_rate = EXCLUDED.win_rate,
        win_streak = EXCLUDED.win_streak,
        loss_streak = EXCLUDED.loss_streak;

