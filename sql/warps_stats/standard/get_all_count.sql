-- Ranked population: players with at least 100 pulls in this pool. The count is
-- stored at recalculation (and backfilled for older rows), so no pull table is scanned.
SELECT uid, luck_4, luck_5, pull_count::bigint AS warp_count
FROM warps_stats_standard
WHERE pull_count >= 100;
