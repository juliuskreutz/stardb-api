INSERT INTO gi_achievements_text (id,
    LANGUAGE, name, description)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::text[], $3::text[], $4::text[])
ON CONFLICT (id,
    LANGUAGE)
    DO UPDATE SET
        name = EXCLUDED.name,
        description = EXCLUDED.description;

