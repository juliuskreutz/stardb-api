SELECT
    zzz_signals.id,
    zzz_signals.uid,
    zzz_signals.gacha_type,
    zzz_signals.character,
    zzz_signals.w_engine,
    zzz_signals.bangboo,
    zzz_signals.timestamp,
    zzz_signals.official,
    COALESCE(zzz_characters_text.name, zzz_w_engines_text.name, zzz_bangboos_text.name) AS name,
    COALESCE(zzz_characters.rarity, zzz_w_engines.rarity, zzz_bangboos.rarity) AS rarity
FROM
    zzz_signals
    LEFT JOIN zzz_characters ON zzz_characters.id = character
    LEFT JOIN zzz_w_engines ON zzz_w_engines.id = w_engine
    LEFT JOIN zzz_bangboos ON zzz_bangboos.id = bangboo
    LEFT JOIN zzz_characters_text ON zzz_characters_text.id = character
        AND zzz_characters_text.language = $3
    LEFT JOIN zzz_w_engines_text ON zzz_w_engines_text.id = w_engine
        AND zzz_w_engines_text.language = $3
    LEFT JOIN zzz_bangboos_text ON zzz_bangboos_text.id = bangboo
        AND zzz_bangboos_text.language = $3
WHERE
    zzz_signals.id = $1
    AND uid = $2;

