DELETE FROM zzz_signals_stats_global_bangboo
WHERE uid = ANY($1::integer[]);
