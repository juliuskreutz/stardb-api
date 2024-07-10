DROP TABLE warps_stats;

DROP TABLE warps_stats_4;

DROP TABLE warps_stats_5;

DROP TYPE gacha_type;

CREATE TABLE IF NOT EXISTS warps_stats_standard (
    uid integer NOT NULL,
    count integer NOT NULL,
    count_rank integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_4_rank integer NOT NULL,
    luck_5 double precision NOT NULL,
    luck_5_rank integer NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_special (
    uid integer NOT NULL,
    count integer NOT NULL,
    count_rank integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_4_rank integer NOT NULL,
    luck_5 double precision NOT NULL,
    luck_5_rank integer NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_lc (
    uid integer NOT NULL,
    count integer NOT NULL,
    count_rank integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_4_rank integer NOT NULL,
    luck_5 double precision NOT NULL,
    luck_5_rank integer NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

ALTER TABLE ONLY warps_stats_standard
    ADD CONSTRAINT warps_stats_standard_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY warps_stats_special
    ADD CONSTRAINT warps_stats_special_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY warps_stats_lc
    ADD CONSTRAINT warps_stats_lc_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY warps_stats_standard
    ADD CONSTRAINT warps_stats_standard_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_stats_special
    ADD CONSTRAINT warps_stats_special_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_stats_lc
    ADD CONSTRAINT warps_stats_lc_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

