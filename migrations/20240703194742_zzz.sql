CREATE TABLE IF NOT EXISTS zzz_achievement_series (
    id integer NOT NULL,
    priority integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_achievement_series_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_achievements (
    id integer NOT NULL,
    series integer NOT NULL,
    polychromes integer NOT NULL,
    hidden boolean NOT NULL,
    version text,
    comment text,
    reference text,
    difficulty text,
    gacha boolean DEFAULT FALSE NOT NULL,
    set integer,
    priority integer NOT NULL,
    video text,
    impossible boolean DEFAULT TRUE NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_achievements_percent (
    id integer NOT NULL,
    percent double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_achievements_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    description text NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_characters (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_characters_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_connections (
    uid integer NOT NULL,
    username text NOT NULL,
    verified boolean NOT NULL,
    private boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_w_engines (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_w_engines_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_uids (
    uid integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_users_achievements_completed (
    username text NOT NULL,
    id integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_users_achievements_favorites (
    username text NOT NULL,
    id integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_signals (
    id bigint NOT NULL,
    uid integer NOT NULL,
    gacha_type text NOT NULL,
    character integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

ALTER TABLE ONLY zzz_achievement_series
    ADD CONSTRAINT zzz_achievement_series_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_achievement_series_text
    ADD CONSTRAINT zzz_achievement_series_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY zzz_achievements_percent
    ADD CONSTRAINT zzz_achievements_percent_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_achievements
    ADD CONSTRAINT zzz_achievements_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_achievements_text
    ADD CONSTRAINT zzz_achievements_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY zzz_characters
    ADD CONSTRAINT zzz_characters_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_characters_text
    ADD CONSTRAINT zzz_characters_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY zzz_connections
    ADD CONSTRAINT zzz_connections_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_w_engines
    ADD CONSTRAINT zzz_w_engines_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_w_engines_text
    ADD CONSTRAINT zzz_w_engines_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY zzz_uids
    ADD CONSTRAINT zzz_uids_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY zzz_users_achievements_completed
    ADD CONSTRAINT zzz_users_achievements_completed_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY zzz_users_achievements_favorites
    ADD CONSTRAINT zzz_users_achievements_favorites_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY zzz_signals
    ADD CONSTRAINT zzz_signals_pkey PRIMARY KEY (id, timestamp);

CREATE INDEX zzz_signals_uid_index ON zzz_signals USING btree (uid);

ALTER TABLE ONLY zzz_achievements_percent
    ADD CONSTRAINT zzz_achievements_percent_id_fkey FOREIGN KEY (id) REFERENCES zzz_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_achievements
    ADD CONSTRAINT zzz_achievements_series_fkey FOREIGN KEY (series) REFERENCES zzz_achievement_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_achievement_series_text
    ADD CONSTRAINT zzz_achievements_series_text_id_fkey FOREIGN KEY (id) REFERENCES zzz_achievement_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_achievements_text
    ADD CONSTRAINT zzz_achievements_text_id_fkey FOREIGN KEY (id) REFERENCES zzz_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_characters_text
    ADD CONSTRAINT zzz_characters_text_id_fkey FOREIGN KEY (id) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_connections
    ADD CONSTRAINT zzz_connections_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_connections
    ADD CONSTRAINT zzz_connections_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_w_engines_text
    ADD CONSTRAINT zzz_w_engines_text_id_fkey FOREIGN KEY (id) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_users_achievements_completed
    ADD CONSTRAINT zzz_users_achievements_completed_id_fkey FOREIGN KEY (id) REFERENCES zzz_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_users_achievements_completed
    ADD CONSTRAINT zzz_users_achievements_completed_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY zzz_users_achievements_favorites
    ADD CONSTRAINT zzz_users_achievements_favorites_id_fkey FOREIGN KEY (id) REFERENCES zzz_achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_users_achievements_favorites
    ADD CONSTRAINT zzz_users_achievements_favorites_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals
    ADD CONSTRAINT zzz_signals_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals
    ADD CONSTRAINT zzz_signals_light_cone_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals
    ADD CONSTRAINT zzz_signals_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

