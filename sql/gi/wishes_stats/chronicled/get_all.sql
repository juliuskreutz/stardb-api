-- Ranked population: players with at least 100 wishes in this pool. The count is
-- stored at recalculation (and backfilled for older rows), so no pull table is scanned.
SELECT uid, luck_4, luck_5, pull_count::bigint AS wish_count
FROM gi_wishes_stats_chronicled
WHERE pull_count >= 100;
