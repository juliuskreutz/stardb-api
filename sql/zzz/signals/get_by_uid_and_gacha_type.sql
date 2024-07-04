SELECT
    zzz_signals.id,
    zzz_signals.uid,
    zzz_signals.gacha_type,
    zzz_signals.character,
    zzz_signals.w_engine,
    zzz_signals.timestamp,
    zzz_signals.official,
    COALESCE(zzz_characters_text.name, zzz_w_engines_text.name) AS name,
    COALESCE(zzz_characters.rarity, zzz_w_engines.rarity) AS rarity
FROM
    zzz_signals
    LEFT JOIN zzz_characters ON zzz_characters.id = character
    LEFT JOIN zzz_w_engines ON zzz_w_engines.id = w_engine
    LEFT JOIN zzz_characters_text ON zzz_characters_text.id = character
        AND zzz_characters_text.language = $3
    LEFT JOIN zzz_w_engines_text ON zzz_w_engines_text.id = w_engine
        AND zzz_w_engines_text.language = $3
WHERE
    uid = $1
    AND gacha_type = $2
ORDER BY
    id;

