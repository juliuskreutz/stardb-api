ALTER TABLE zzz_signals_stats_standard
    DROP COLUMN count_percentile;

ALTER TABLE zzz_signals_stats_standard
    DROP COLUMN luck_a_percentile;

ALTER TABLE zzz_signals_stats_standard
    DROP COLUMN luck_s_percentile;

ALTER TABLE zzz_signals_stats_special
    DROP COLUMN count_percentile;

ALTER TABLE zzz_signals_stats_special
    DROP COLUMN luck_a_percentile;

ALTER TABLE zzz_signals_stats_special
    DROP COLUMN luck_s_percentile;

ALTER TABLE zzz_signals_stats_w_engine
    DROP COLUMN count_percentile;

ALTER TABLE zzz_signals_stats_w_engine
    DROP COLUMN luck_a_percentile;

ALTER TABLE zzz_signals_stats_w_engine
    DROP COLUMN luck_s_percentile;

ALTER TABLE zzz_signals_stats_bangboo
    DROP COLUMN count_percentile;

ALTER TABLE zzz_signals_stats_bangboo
    DROP COLUMN luck_a_percentile;

ALTER TABLE zzz_signals_stats_bangboo
    DROP COLUMN luck_s_percentile;

CREATE TABLE IF NOT EXISTS zzz_signals_stats_global_standard (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals_stats_global_special (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals_stats_global_w_engine (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals_stats_global_bangboo (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);

ALTER TABLE ONLY zzz_signals_stats_global_standard
    ADD CONSTRAINT zzz_signals_stats_global_standard_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_global_special
    ADD CONSTRAINT zzz_signals_stats_global_special_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_global_w_engine
    ADD CONSTRAINT zzz_signals_stats_global_w_engine_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_global_bangboo
    ADD CONSTRAINT zzz_signals_stats_global_bangboo_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_global_standard
    ADD CONSTRAINT zzz_signals_stats_global_standard_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_stats_global_special
    ADD CONSTRAINT zzz_signals_stats_global_special_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_stats_global_w_engine
    ADD CONSTRAINT zzz_signals_stats_global_w_engine_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_stats_global_bangboo
    ADD CONSTRAINT zzz_signals_stats_global_bangboo_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

