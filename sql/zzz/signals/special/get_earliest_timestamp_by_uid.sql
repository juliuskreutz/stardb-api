SELECT MIN(timestamp) AS "timestamp?"
FROM zzz_signals_special
WHERE uid = $1;
