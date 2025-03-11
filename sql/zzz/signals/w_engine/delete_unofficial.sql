DELETE FROM zzz_signals_w_engine
WHERE uid = $1
    AND NOT official;

