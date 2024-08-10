INSERT INTO light_cones_text (id,
    LANGUAGE, name, path)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::text[], $3::text[], $4::text[])
ON CONFLICT (id,
    LANGUAGE)
    DO UPDATE SET
        name = EXCLUDED.name,
        path = EXCLUDED.path;

