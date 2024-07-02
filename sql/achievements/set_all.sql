INSERT INTO achievements (id, series, jades, hidden, priority)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::integer[], $3::integer[], $4::bool[], $5::integer[])
ON CONFLICT (id)
    DO UPDATE SET
        series = EXCLUDED.series,
        jades = EXCLUDED.jades,
        hidden = EXCLUDED.hidden,
        priority = EXCLUDED.priority;

