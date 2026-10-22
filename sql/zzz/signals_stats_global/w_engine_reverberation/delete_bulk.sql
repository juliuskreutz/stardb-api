DELETE FROM zzz_signals_stats_global_w_engine_reverberation
WHERE uid = ANY($1::integer[]);
