WITH signal_counts AS (
    SELECT uid, COUNT(*) AS signal_count
    FROM zzz_signals_bangboo
    GROUP BY uid
)
-- The preserved side is NOT NULL even if the planner reverses the LEFT JOIN.
-- Pin SQLx nullability; only a missing history count is optional.
SELECT stats.uid AS "uid!", stats.luck_a AS "luck_a!", stats.luck_s AS "luck_s!", counts.signal_count AS "signal_count?"
FROM zzz_signals_stats_bangboo AS stats
LEFT JOIN signal_counts AS counts USING (uid);
