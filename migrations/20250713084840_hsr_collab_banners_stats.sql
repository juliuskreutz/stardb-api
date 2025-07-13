CREATE TABLE IF NOT EXISTS warps_stats_collab_lc
(
    uid         INTEGER NOT NULL,
    luck_4      DOUBLE PRECISION NOT NULL,
    luck_5      DOUBLE PRECISION NOT NULL,
    win_rate    DOUBLE PRECISION NOT NULL,
    win_streak  INTEGER NOT NULL,
    loss_streak INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_collab
(
    uid         INTEGER NOT NULL,
    luck_4      DOUBLE PRECISION NOT NULL,
    luck_5      DOUBLE PRECISION NOT NULL,
    win_rate    DOUBLE PRECISION NOT NULL,
    win_streak  INTEGER NOT NULL,
    loss_streak INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_global_collab_lc
(
    uid               INTEGER NOT NULL,
    count_percentile  DOUBLE PRECISION NOT NULL,
    luck_4_percentile DOUBLE PRECISION NOT NULL,
    luck_5_percentile DOUBLE PRECISION NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_global_collab
(
    uid               INTEGER NOT NULL,
    count_percentile  DOUBLE PRECISION NOT NULL,
    luck_4_percentile DOUBLE PRECISION NOT NULL,
    luck_5_percentile DOUBLE PRECISION NOT NULL
);

ALTER TABLE warps_stats_global_collab ADD CONSTRAINT warps_stats_global_collab_pkey PRIMARY KEY (uid);

ALTER TABLE warps_stats_global_collab ADD CONSTRAINT warps_stats_global_collab_uid_fkey
    FOREIGN KEY (uid) REFERENCES mihomo ON DELETE CASCADE;

ALTER TABLE warps_stats_global_collab_lc ADD CONSTRAINT warps_stats_global_collab_lc_pkey PRIMARY KEY (uid);

ALTER TABLE warps_stats_global_collab_lc ADD CONSTRAINT warps_stats_global_collab_lc_uid_fkey
    FOREIGN KEY (uid) REFERENCES mihomo ON DELETE CASCADE;

ALTER TABLE warps_stats_collab ADD CONSTRAINT warps_stats_collab_pkey PRIMARY KEY (uid);

ALTER TABLE warps_stats_collab ADD CONSTRAINT warps_stats_collab_uid_fkey
    FOREIGN KEY (uid) REFERENCES mihomo ON DELETE CASCADE;

ALTER TABLE warps_stats_collab_lc ADD CONSTRAINT warps_stats_collab_lc_pkey PRIMARY KEY (uid);

ALTER TABLE warps_stats_collab_lc ADD CONSTRAINT warps_stats_collab_lc_uid_fkey
    FOREIGN KEY (uid) REFERENCES mihomo ON DELETE CASCADE;