INSERT INTO gi_wishes_stats_global_weapon (uid, count_percentile, luck_4_percentile, luck_5_percentile)
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
    luck_4_percentile = EXCLUDED.luck_4_percentile,
    luck_5_percentile = EXCLUDED.luck_5_percentile
-- Identical hourly results need no new heap tuple; changed percentiles still update.
WHERE (gi_wishes_stats_global_weapon.count_percentile, gi_wishes_stats_global_weapon.luck_4_percentile, gi_wishes_stats_global_weapon.luck_5_percentile)
    IS DISTINCT FROM (EXCLUDED.count_percentile, EXCLUDED.luck_4_percentile, EXCLUDED.luck_5_percentile);
