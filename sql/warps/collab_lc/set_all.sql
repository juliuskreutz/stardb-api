INSERT INTO warps_collab_lc (id, uid, character, light_cone, timestamp, official)
SELECT
    *
FROM
    UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[], $5::timestamp[], $6::boolean[])
ON CONFLICT (uid, id) DO UPDATE SET
    character = EXCLUDED.character,
    light_cone = EXCLUDED.light_cone,
    timestamp = EXCLUDED.timestamp,
    official = TRUE
WHERE EXCLUDED.official AND NOT warps_collab_lc.official;

