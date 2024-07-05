INSERT INTO zzz_signals (id, uid, gacha_type, character, w_engine, bangboo, timestamp, official)
SELECT
    *
FROM
    UNNEST($1::bigint[], $2::integer[], $3::text[], $4::integer[], $5::integer[], $6::integer[], $7::timestamp[], $8::boolean[])
ON CONFLICT
    DO NOTHING;

