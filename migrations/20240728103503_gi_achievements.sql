CREATE TABLE IF NOT EXISTS gi_achievement_series (
    id integer NOT NULL,
    priority integer NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_achievement_series_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_achievements (
    id integer NOT NULL,
    series integer NOT NULL,
    primogems integer NOT NULL,
    hidden boolean NOT NULL,
    version text,
    comment text,
    reference text,
    difficulty text,
    gacha boolean DEFAULT FALSE NOT NULL,
    timegated boolean DEFAULT FALSE NOT NULL,
    missable boolean DEFAULT FALSE NOT NULL,
    set integer,
    priority integer NOT NULL,
    video text,
    impossible boolean DEFAULT FALSE NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_achievements_percent (
    id integer NOT NULL,
    percent double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_achievements_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    description text NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_users_achievements_completed (
    username text NOT NULL,
    id integer NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_users_achievements_favorites (
    username text NOT NULL,
    id integer NOT NULL
);

ALTER TABLE ONLY gi_achievement_series
    ADD CONSTRAINT gi_achievement_series_pkey PRIMARY KEY (id);

ALTER TABLE ONLY gi_achievement_series_text
    ADD CONSTRAINT gi_achievement_series_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY gi_achievements_percent
    ADD CONSTRAINT gi_achievements_percent_pkey PRIMARY KEY (id);

ALTER TABLE ONLY gi_achievements
    ADD CONSTRAINT gi_achievements_pkey PRIMARY KEY (id);

ALTER TABLE ONLY gi_achievements_text
    ADD CONSTRAINT gi_achievements_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY gi_users_achievements_completed
    ADD CONSTRAINT gi_users_achievements_completed_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY gi_users_achievements_favorites
    ADD CONSTRAINT gi_users_achievements_favorites_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY gi_achievements_percent
    ADD CONSTRAINT gi_achievements_percent_id_fkey FOREIGN KEY (id) REFERENCES gi_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_achievements
    ADD CONSTRAINT gi_achievements_series_fkey FOREIGN KEY (series) REFERENCES gi_achievement_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_achievement_series_text
    ADD CONSTRAINT gi_achievements_series_text_id_fkey FOREIGN KEY (id) REFERENCES gi_achievement_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_achievements_text
    ADD CONSTRAINT gi_achievements_text_id_fkey FOREIGN KEY (id) REFERENCES gi_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_users_achievements_completed
    ADD CONSTRAINT gi_users_achievements_completed_id_fkey FOREIGN KEY (id) REFERENCES gi_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_users_achievements_completed
    ADD CONSTRAINT gi_users_achievements_completed_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_users_achievements_favorites
    ADD CONSTRAINT gi_users_achievements_favorites_id_fkey FOREIGN KEY (id) REFERENCES gi_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_users_achievements_favorites
    ADD CONSTRAINT gi_users_achievements_favorites_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

