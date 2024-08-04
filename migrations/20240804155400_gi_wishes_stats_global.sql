CREATE TABLE IF NOT EXISTS gi_wishes_stats_global_standard (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_stats_global_character (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_stats_global_weapon (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_stats_global_chronicled (
    uid integer NOT NULL,
    count_percentile double precision NOT NULL,
    luck_4_percentile double precision NOT NULL,
    luck_5_percentile double precision NOT NULL
);

