SELECT
    id,
    uid
FROM
    zzz_signals_standard
WHERE
    id = $1
    AND uid = $2;

