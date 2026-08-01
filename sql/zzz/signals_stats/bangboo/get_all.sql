WITH signal_counts AS (
    SELECT uid, COUNT(*) AS signal_count
    FROM zzz_signals_bangboo
    GROUP BY uid
)
SELECT stats.uid, stats.luck_a, stats.luck_s, counts.signal_count
FROM zzz_signals_stats_bangboo AS stats
LEFT JOIN signal_counts AS counts USING (uid);
