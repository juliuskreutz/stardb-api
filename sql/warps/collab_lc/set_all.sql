INSERT INTO warps_collab_lc (id, uid, character, light_cone, timestamp, official)
SELECT
    *
FROM
    UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[], $5::timestamp[], $6::boolean[])
ON CONFLICT
    DO NOTHING;

