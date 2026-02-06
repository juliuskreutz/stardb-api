DELETE FROM zzz_signals_exclusive_rescreening
WHERE uid = $1
    AND NOT official;

