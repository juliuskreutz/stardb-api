-- Store each player's pull count with their per-pool stats. The hourly global
-- updaters previously recounted every pull table (about 300M rows) to rank
-- players, saturating the database. Recalculation writes the count from the
-- rows it already reads; NULL marks rows from before this column, which the
-- updaters backfill in small chunks. Nullable without a default, so adding
-- the column does not rewrite these tables.

ALTER TABLE warps_stats_standard ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE warps_stats_special ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE warps_stats_lc ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE warps_stats_collab ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE warps_stats_collab_lc ADD COLUMN pull_count integer CHECK (pull_count >= 0);

ALTER TABLE gi_wishes_stats_standard ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE gi_wishes_stats_character ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE gi_wishes_stats_weapon ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE gi_wishes_stats_chronicled ADD COLUMN pull_count integer CHECK (pull_count >= 0);

ALTER TABLE zzz_signals_stats_standard ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE zzz_signals_stats_special ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE zzz_signals_stats_w_engine ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE zzz_signals_stats_bangboo ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE zzz_signals_stats_exclusive_rescreening ADD COLUMN pull_count integer CHECK (pull_count >= 0);
ALTER TABLE zzz_signals_stats_w_engine_reverberation ADD COLUMN pull_count integer CHECK (pull_count >= 0);
