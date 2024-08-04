CREATE TABLE IF NOT EXISTS gi_wishes_stats_standard (
    uid integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_5 double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_stats_character (
    uid integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_5 double precision NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_stats_weapon (
    uid integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_5 double precision NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_stats_chronicled (
    uid integer NOT NULL,
    luck_4 double precision NOT NULL,
    luck_5 double precision NOT NULL
);

ALTER TABLE ONLY gi_wishes_stats_standard
    ADD CONSTRAINT gi_wishes_stats_standard_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_stats_character
    ADD CONSTRAINT gi_wishes_stats_character_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_stats_weapon
    ADD CONSTRAINT gi_wishes_stats_weapon_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_stats_chronicled
    ADD CONSTRAINT gi_wishes_stats_chronicled_pkey PRIMARY KEY (uid);

