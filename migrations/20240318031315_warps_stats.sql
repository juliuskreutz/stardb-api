CREATE TABLE IF NOT EXISTS warps_stats (
    uid INT8 NOT NULL,
    gacha_type TEXT NOT NULL,
    COUNT INT4 NOT NULL,
    RANK INT4 NOT NULL,
    PRIMARY KEY (uid, gacha_type)
);

CREATE TABLE IF NOT EXISTS warps_stats_4 (
    uid INT8 NOT NULL,
    gacha_type TEXT NOT NULL,
    COUNT INT4 NOT NULL,
    AVG FLOAT8 NOT NULL,
    rank_count INT4 NOT NULL,
    rank_avg INT4 NOT NULL,
    PRIMARY KEY (uid, gacha_type)
);

CREATE TABLE IF NOT EXISTS warps_stats_5 (
    uid INT8 NOT NULL,
    gacha_type TEXT NOT NULL,
    COUNT INT4 NOT NULL,
    AVG FLOAT8 NOT NULL,
    rank_count INT4 NOT NULL,
    rank_avg INT4 NOT NULL,
    PRIMARY KEY (uid, gacha_type)
);
