SELECT
    zzz_signals_bangboo.id,
    zzz_signals_bangboo.uid,
    NULL::integer AS character,
    zzz_signals_bangboo.bangboo,
    zzz_signals_bangboo.w_engine,
    zzz_signals_bangboo.timestamp,
    zzz_signals_bangboo.official,
    COALESCE(zzz_bangboos_text.name, zzz_w_engines_text.name) AS name,
    COALESCE(zzz_bangboos.rarity, zzz_w_engines.rarity) AS rarity
FROM
    zzz_signals_bangboo
    LEFT JOIN zzz_bangboos ON zzz_bangboos.id = bangboo
    LEFT JOIN zzz_w_engines ON zzz_w_engines.id = w_engine
    LEFT JOIN zzz_bangboos_text ON zzz_bangboos_text.id = bangboo
        AND zzz_bangboos_text.language = $2
    LEFT JOIN zzz_w_engines_text ON zzz_w_engines_text.id = w_engine
        AND zzz_w_engines_text.language = $2
WHERE
    uid = $1
ORDER BY
    id;

