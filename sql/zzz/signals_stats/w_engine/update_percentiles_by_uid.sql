UPDATE
    zzz_signals_stats_w_engine
SET
    count_percentile = $2,
    luck_a_percentile = $3,
    luck_s_percentile = $4
WHERE
    uid = $1;

