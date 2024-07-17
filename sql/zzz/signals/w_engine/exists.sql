SELECT
    id,
    uid
FROM
    zzz_signals_w_engine
WHERE
    id = $1
    AND uid = $2;

