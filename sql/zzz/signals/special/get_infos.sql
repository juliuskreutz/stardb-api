SELECT
    zzz_signals_special.uid,
    zzz_signals_special.character,
    NULL::integer AS bangboo,
    zzz_signals_special.w_engine,
    COALESCE(zzz_characters.rarity, zzz_w_engines.rarity) AS rarity
FROM
    zzz_signals_special
    LEFT JOIN zzz_characters ON zzz_characters.id = character
    LEFT JOIN zzz_w_engines ON zzz_w_engines.id = w_engine
WHERE
    uid = $1
ORDER BY
    zzz_signals_special.id;

