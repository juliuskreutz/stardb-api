WITH signal_counts AS (
    SELECT uid, COUNT(*) AS signal_count
    FROM zzz_signals_exclusive_rescreening
    GROUP BY uid
)
SELECT stats.uid, stats.luck_a, stats.luck_s, counts.signal_count
FROM zzz_signals_stats_exclusive_rescreening AS stats
LEFT JOIN signal_counts AS counts USING (uid);
