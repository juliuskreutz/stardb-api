DELETE FROM zzz_signals_bangboo
WHERE uid = $1
    AND NOT official;

