SELECT
    NULL::integer AS character,
    zzz_signals_bangboo.w_engine,
    COALESCE(zzz_bangboos.rarity, zzz_w_engines.rarity) AS rarity
FROM
    zzz_signals_bangboo
    LEFT JOIN zzz_bangboos ON zzz_bangboos.id = bangboo
    LEFT JOIN zzz_w_engines ON zzz_w_engines.id = w_engine
WHERE
    uid = $1
ORDER BY
    zzz_signals_bangboo.id;

