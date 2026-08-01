SELECT MIN(timestamp) AS "timestamp?"
FROM zzz_signals_standard
WHERE uid = $1;
