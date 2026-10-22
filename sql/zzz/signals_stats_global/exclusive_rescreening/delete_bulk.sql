DELETE FROM zzz_signals_stats_global_exclusive_rescreening
WHERE uid = ANY($1::integer[]);
