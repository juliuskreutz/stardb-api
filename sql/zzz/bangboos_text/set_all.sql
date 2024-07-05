INSERT INTO zzz_bangboos_text (id,
    LANGUAGE, name)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::text[], $3::text[])
ON CONFLICT (id,
    LANGUAGE)
    DO UPDATE SET
        name = EXCLUDED.name;

