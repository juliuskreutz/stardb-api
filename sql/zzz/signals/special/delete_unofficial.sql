DELETE FROM zzz_signals_special
WHERE uid = $1
    AND NOT official;

