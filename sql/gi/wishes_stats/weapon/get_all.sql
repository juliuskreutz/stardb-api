WITH wish_counts AS (
    SELECT uid, COUNT(*) AS wish_count
    FROM gi_wishes_weapon
    GROUP BY uid
)
SELECT stats.uid, stats.luck_4, stats.luck_5, counts.wish_count
FROM gi_wishes_stats_weapon AS stats
JOIN wish_counts AS counts USING (uid)
WHERE counts.wish_count >= 100;
