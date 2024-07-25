INSERT INTO gi_wishes_character (id, uid, character, weapon, timestamp, official)
SELECT
    *
FROM
    UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[], $5::timestamp[], $6::boolean[])
ON CONFLICT
    DO NOTHING;

