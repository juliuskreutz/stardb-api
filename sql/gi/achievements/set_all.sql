INSERT INTO gi_achievements (id, series, primogems, hidden, priority)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::integer[], $3::integer[], $4::bool[], $5::integer[])
ON CONFLICT (id)
    DO UPDATE SET
        series = EXCLUDED.series,
        primogems = EXCLUDED.primogems,
        hidden = EXCLUDED.hidden,
        priority = EXCLUDED.priority;

