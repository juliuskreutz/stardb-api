UPDATE
    warps_stats_standard
SET
    count_percentile = $2,
    luck_4_percentile = $3,
    luck_5_percentile = $4
WHERE
    uid = $1;

