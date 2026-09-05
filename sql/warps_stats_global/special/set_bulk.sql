INSERT INTO warps_stats_global_special (uid, count_percentile, luck_4_percentile, luck_5_percentile)
SELECT *
FROM UNNEST(
    $1::integer[],         -- uid
    $2::double precision[],-- count_percentile
    $3::double precision[],-- luck_4_percentile
    $4::double precision[] -- luck_5_percentile
)
ON CONFLICT (uid)
DO UPDATE SET
    count_percentile = EXCLUDED.count_percentile,
    luck_4_percentile = EXCLUDED.luck_4_percentile,
    luck_5_percentile = EXCLUDED.luck_5_percentile
-- Identical hourly results need no new heap tuple; changed percentiles still update.
WHERE (warps_stats_global_special.count_percentile, warps_stats_global_special.luck_4_percentile, warps_stats_global_special.luck_5_percentile)
    IS DISTINCT FROM (EXCLUDED.count_percentile, EXCLUDED.luck_4_percentile, EXCLUDED.luck_5_percentile);
