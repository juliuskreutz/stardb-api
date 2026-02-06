DELETE FROM zzz_signals_w_engine_reverberation
WHERE uid = $1
    AND NOT official;

