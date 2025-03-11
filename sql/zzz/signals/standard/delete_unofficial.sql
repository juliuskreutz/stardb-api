DELETE FROM zzz_signals_standard
WHERE uid = $1
    AND NOT official;

