INSERT INTO achievements (id, series, jades, hidden, priority)
    VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id)
    DO UPDATE SET
        series = EXCLUDED.series, jades = EXCLUDED.jades, hidden = EXCLUDED.hidden, priority = EXCLUDED.priority;

