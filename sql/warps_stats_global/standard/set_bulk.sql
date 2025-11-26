INSERT INTO warps_stats_global_standard (uid, count_percentile, luck_4_percentile, luck_5_percentile)
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
    luck_5_percentile = EXCLUDED.luck_5_percentile;