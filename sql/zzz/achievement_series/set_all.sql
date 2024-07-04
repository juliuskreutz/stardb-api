INSERT INTO zzz_achievement_series (id, priority)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::integer[])
ON CONFLICT (id)
    DO UPDATE SET
        priority = EXCLUDED.priority;

