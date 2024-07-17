SELECT
    zzz_signals_special.id,
    zzz_signals_special.uid,
    zzz_signals_special.character,
    NULL::integer AS bangboo,
    zzz_signals_special.w_engine,
    zzz_signals_special.timestamp,
    zzz_signals_special.official,
    COALESCE(zzz_characters_text.name, zzz_w_engines_text.name) AS name,
    COALESCE(zzz_characters.rarity, zzz_w_engines.rarity) AS rarity
FROM
    zzz_signals_special
    LEFT JOIN zzz_characters ON zzz_characters.id = character
    LEFT JOIN zzz_w_engines ON zzz_w_engines.id = w_engine
    LEFT JOIN zzz_characters_text ON zzz_characters_text.id = character
        AND zzz_characters_text.language = $2
    LEFT JOIN zzz_w_engines_text ON zzz_w_engines_text.id = w_engine
        AND zzz_w_engines_text.language = $2
WHERE
    uid = $1
ORDER BY
    id;

