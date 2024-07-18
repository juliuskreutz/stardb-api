ALTER TABLE warps_stats_standard
    DROP COLUMN count_percentile;

ALTER TABLE warps_stats_standard
    DROP COLUMN luck_4_percentile;

ALTER TABLE warps_stats_standard
    DROP COLUMN luck_5_percentile;

ALTER TABLE warps_stats_special
    DROP COLUMN count_percentile;

ALTER TABLE warps_stats_special
    DROP COLUMN luck_4_percentile;

ALTER TABLE warps_stats_special
    DROP COLUMN luck_5_percentile;

ALTER TABLE warps_stats_lc
    DROP COLUMN count_percentile;

ALTER TABLE warps_stats_lc
    DROP COLUMN luck_4_percentile;

ALTER TABLE warps_stats_lc
    DROP COLUMN luck_5_percentile;

CREATE TABLE IF NOT EXISTS warps_stats_global_standard (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_global_special (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_global_lc (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

ALTER TABLE ONLY warps_stats_global_standard
    ADD CONSTRAINT warps_stats_global_standard_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY warps_stats_global_special
    ADD CONSTRAINT warps_stats_global_special_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY warps_stats_global_lc
    ADD CONSTRAINT warps_stats_global_lc_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY warps_stats_global_standard
    ADD CONSTRAINT warps_stats_global_standard_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_stats_global_special
    ADD CONSTRAINT warps_stats_global_special_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_stats_global_lc
    ADD CONSTRAINT warps_stats_global_lc_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

