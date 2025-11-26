WITH warp_counts AS (
    SELECT uid, COUNT(*) AS warp_count
    FROM warps_standard
    GROUP BY uid
)
SELECT
    stats.uid,
    stats.luck_4,
    stats.luck_5,
    COALESCE(wc.warp_count, 0) AS warp_count
FROM warps_stats_standard stats
         LEFT JOIN warp_counts wc ON stats.uid = wc.uid
WHERE wc.warp_count >= 100;