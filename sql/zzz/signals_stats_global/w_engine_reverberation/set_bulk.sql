INSERT INTO zzz_signals_stats_global_w_engine_reverberation (uid, count_percentile, luck_a_percentile, luck_s_percentile)
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
    luck_s_percentile = EXCLUDED.luck_s_percentile
-- Identical hourly results need no new heap tuple; changed percentiles still update.
WHERE (zzz_signals_stats_global_w_engine_reverberation.count_percentile, zzz_signals_stats_global_w_engine_reverberation.luck_a_percentile, zzz_signals_stats_global_w_engine_reverberation.luck_s_percentile)
    IS DISTINCT FROM (EXCLUDED.count_percentile, EXCLUDED.luck_a_percentile, EXCLUDED.luck_s_percentile);
