INSERT INTO gi_wishes_standard (id, uid, character, weapon, timestamp, official)
SELECT
    *
FROM
    UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[], $5::timestamp[], $6::boolean[])
ON CONFLICT (uid, id) DO UPDATE SET
    character = EXCLUDED.character,
    weapon = EXCLUDED.weapon,
    timestamp = EXCLUDED.timestamp,
    official = TRUE
WHERE EXCLUDED.official AND NOT gi_wishes_standard.official;

