INSERT INTO zzz_signals_w_engine (id, uid, character, w_engine, timestamp, official)
SELECT
    *
FROM
    UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[], $5::timestamp[], $6::boolean[])
ON CONFLICT (uid, id) DO UPDATE SET
    character = EXCLUDED.character,
    w_engine = EXCLUDED.w_engine,
    timestamp = EXCLUDED.timestamp,
    official = TRUE
WHERE EXCLUDED.official AND NOT zzz_signals_w_engine.official;

