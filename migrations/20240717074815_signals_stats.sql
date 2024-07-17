CREATE TABLE IF NOT EXISTS zzz_signals_stats_standard (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals_stats_special (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s double precision NOT NULL,
    luck_s_percentile double precision NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals_stats_w_engine (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s double precision NOT NULL,
    luck_s_percentile double precision NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals_stats_bangboo (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_a double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);

ALTER TABLE ONLY zzz_signals_stats_standard
    ADD CONSTRAINT zzz_signals_stats_standard_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_special
    ADD CONSTRAINT zzz_signals_stats_special_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_w_engine
    ADD CONSTRAINT zzz_signals_stats_w_engine_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_bangboo
    ADD CONSTRAINT zzz_signals_stats_bangboo_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_signals_stats_standard
    ADD CONSTRAINT zzz_signals_stats_standard_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_stats_special
    ADD CONSTRAINT zzz_signals_stats_special_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_stats_w_engine
    ADD CONSTRAINT zzz_signals_stats_w_engine_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_stats_bangboo
    ADD CONSTRAINT zzz_signals_stats_bangboo_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

