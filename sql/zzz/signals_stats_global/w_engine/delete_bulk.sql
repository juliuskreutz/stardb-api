DELETE FROM zzz_signals_stats_global_w_engine
WHERE uid = ANY($1::integer[]);
