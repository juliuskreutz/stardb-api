ALTER TABLE warps_stats_standard
    ALTER COLUMN count_rank TYPE double precision;

ALTER TABLE warps_stats_standard
    ALTER COLUMN luck_4_rank TYPE double precision;

ALTER TABLE warps_stats_standard
    ALTER COLUMN luck_5_rank TYPE double precision;

ALTER TABLE warps_stats_special
    ALTER COLUMN count_rank TYPE double precision;

ALTER TABLE warps_stats_special
    ALTER COLUMN luck_4_rank TYPE double precision;

ALTER TABLE warps_stats_special
    ALTER COLUMN luck_5_rank TYPE double precision;

ALTER TABLE warps_stats_lc
    ALTER COLUMN count_rank TYPE double precision;

ALTER TABLE warps_stats_lc
    ALTER COLUMN luck_4_rank TYPE double precision;

ALTER TABLE warps_stats_lc
    ALTER COLUMN luck_5_rank TYPE double precision;

ALTER TABLE warps_stats_standard RENAME COLUMN count_rank TO count_percentile;

ALTER TABLE warps_stats_special RENAME COLUMN count_rank TO count_percentile;

ALTER TABLE warps_stats_lc RENAME COLUMN count_rank TO count_percentile;

ALTER TABLE warps_stats_standard RENAME COLUMN luck_4_rank TO luck_4_percentile;

ALTER TABLE warps_stats_standard RENAME COLUMN luck_5_rank TO luck_5_percentile;

ALTER TABLE warps_stats_special RENAME COLUMN luck_4_rank TO luck_4_percentile;

ALTER TABLE warps_stats_special RENAME COLUMN luck_5_rank TO luck_5_percentile;

ALTER TABLE warps_stats_lc RENAME COLUMN luck_4_rank TO luck_4_percentile;

ALTER TABLE warps_stats_lc RENAME COLUMN luck_5_rank TO luck_5_percentile;

