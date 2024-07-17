SELECT
    id,
    uid
FROM
    zzz_signals_special
WHERE
    id = $1
    AND uid = $2;

