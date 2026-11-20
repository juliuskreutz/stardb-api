-- Every stats row with its stored signal count; the updater applies eligibility.
-- The count is stored at recalculation (and backfilled for older rows), so no
-- pull table is scanned.
SELECT uid AS "uid!", luck_a AS "luck_a!", luck_s AS "luck_s!", pull_count::bigint AS "signal_count?"
FROM zzz_signals_stats_special;
