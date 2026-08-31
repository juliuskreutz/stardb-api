INSERT INTO zzz_signals_stats_global_w_engine (uid, count_percentile, luck_a_percentile, luck_s_percentile)
SELECT *
FROM UNNEST(
    $1::integer[],
    $2::double precision[],
    $3::double precision[],
    $4::double precision[]
)
ON CONFLICT (uid)
DO UPDATE SET
    count_percentile = EXCLUDED.count_percentile,
    luck_a_percentile = EXCLUDED.luck_a_percentile,
    luck_s_percentile = EXCLUDED.luck_s_percentile;
