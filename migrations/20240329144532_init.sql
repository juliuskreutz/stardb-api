CREATE EXTENSION IF NOT EXISTS fuzzystrmatch WITH SCHEMA public;

CREATE TABLE IF NOT EXISTS achievement_series (
    id integer NOT NULL,
    priority integer NOT NULL
);

CREATE TABLE IF NOT EXISTS achievement_series_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS achievements (
    id bigint NOT NULL,
    series integer NOT NULL,
    jades integer NOT NULL,
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

CREATE TABLE IF NOT EXISTS achievements_percent (
    id bigint NOT NULL,
    percent double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS achievements_text (
    id bigint NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    description text NOT NULL
);

CREATE TABLE IF NOT EXISTS admins (
    username text NOT NULL
);

CREATE TABLE IF NOT EXISTS book_series (
    id integer NOT NULL,
    world integer NOT NULL
);

CREATE TABLE IF NOT EXISTS book_series_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS book_series_worlds (
    id integer NOT NULL
);

CREATE TABLE IF NOT EXISTS book_series_worlds_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS books (
    id bigint NOT NULL,
    series_inside integer NOT NULL,
    series integer NOT NULL,
    comment text,
    image1 text,
    image2 text,
    icon integer
);

CREATE TABLE IF NOT EXISTS books_percent (
    id bigint NOT NULL,
    percent double precision NOT NULL
);

CREATE TABLE IF NOT EXISTS books_text (
    id bigint NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS characters (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS characters_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    element text NOT NULL,
    path text NOT NULL
);

CREATE TABLE IF NOT EXISTS community_tier_list_entries (
    character integer NOT NULL,
    eidolon integer NOT NULL,
    average double precision NOT NULL,
    variance double precision NOT NULL,
    votes integer NOT NULL,
    total_votes integer DEFAULT 0 NOT NULL,
    quartile_1 double precision DEFAULT 0 NOT NULL,
    quartile_3 double precision DEFAULT 0 NOT NULL,
    confidence_interval_95 double precision DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS community_tier_list_sextiles (
    value double precision NOT NULL,
    id integer NOT NULL
);

CREATE TABLE IF NOT EXISTS connections (
    uid bigint NOT NULL,
    username text NOT NULL
);

CREATE TABLE IF NOT EXISTS light_cones (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS light_cones_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS mihomo (
    uid bigint NOT NULL,
    region text NOT NULL,
    name text NOT NULL,
    level integer NOT NULL,
    signature text NOT NULL,
    avatar_icon text NOT NULL,
    achievement_count integer NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE IF NOT EXISTS scores_achievement (
    uid bigint NOT NULL,
    timestamp timestamp with time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    uuid uuid NOT NULL,
    username text NOT NULL,
    expiry timestamp with time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS skills (
    id integer NOT NULL,
    character integer NOT NULL
);

CREATE TABLE IF NOT EXISTS skills_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    username text NOT NULL,
    password text NOT NULL,
    email text
);

CREATE TABLE IF NOT EXISTS users_achievements (
    username text NOT NULL,
    id bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS users_books (
    username text NOT NULL,
    id bigint NOT NULL
);

CREATE TABLE IF NOT EXISTS warps (
    id bigint NOT NULL,
    uid bigint NOT NULL,
    gacha_type text NOT NULL,
    character integer,
    light_cone integer,
    timestamp timestamp with time zone NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats (
    uid bigint NOT NULL,
    gacha_type text NOT NULL,
    count integer NOT NULL,
    rank integer NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_4 (
    uid bigint NOT NULL,
    gacha_type text NOT NULL,
    count integer NOT NULL,
    avg double precision NOT NULL,
    rank_count integer NOT NULL,
    rank_avg integer NOT NULL,
    median integer DEFAULT 0 NOT NULL,
    rank_median integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_stats_5 (
    uid bigint NOT NULL,
    gacha_type text NOT NULL,
    count integer NOT NULL,
    avg double precision NOT NULL,
    rank_count integer NOT NULL,
    rank_avg integer NOT NULL,
    median integer DEFAULT 0 NOT NULL,
    rank_median integer DEFAULT 0 NOT NULL
);

ALTER TABLE ONLY achievements_percent
    ADD CONSTRAINT achievements_percent_pkey PRIMARY KEY (id);

ALTER TABLE ONLY achievements
    ADD CONSTRAINT achievements_pkey PRIMARY KEY (id);

ALTER TABLE ONLY achievements_text
    ADD CONSTRAINT achievements_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY admins
    ADD CONSTRAINT admins_pkey PRIMARY KEY (username);

ALTER TABLE ONLY book_series
    ADD CONSTRAINT book_series_pkey PRIMARY KEY (id);

ALTER TABLE ONLY book_series_text
    ADD CONSTRAINT book_series_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY book_series_worlds
    ADD CONSTRAINT book_series_worlds_pkey PRIMARY KEY (id);

ALTER TABLE ONLY book_series_worlds_text
    ADD CONSTRAINT book_series_worlds_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY books_percent
    ADD CONSTRAINT books_percent_pkey PRIMARY KEY (id);

ALTER TABLE ONLY books
    ADD CONSTRAINT books_pkey PRIMARY KEY (id);

ALTER TABLE ONLY books_text
    ADD CONSTRAINT books_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY characters
    ADD CONSTRAINT characters_pkey PRIMARY KEY (id);

ALTER TABLE ONLY characters_text
    ADD CONSTRAINT characters_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY community_tier_list_entries
    ADD CONSTRAINT community_tier_list_entries_pkey PRIMARY KEY (character, eidolon);

ALTER TABLE ONLY community_tier_list_sextiles
    ADD CONSTRAINT community_tier_list_sextiles_pkey PRIMARY KEY (id);

ALTER TABLE ONLY users_achievements
    ADD CONSTRAINT users_achievements_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY connections
    ADD CONSTRAINT connections_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY light_cones
    ADD CONSTRAINT light_cones_pkey PRIMARY KEY (id);

ALTER TABLE ONLY light_cones_text
    ADD CONSTRAINT light_cones_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY mihomo
    ADD CONSTRAINT mihomo_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY scores_achievement
    ADD CONSTRAINT scores_achievement_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY achievement_series
    ADD CONSTRAINT achievement_series_pkey PRIMARY KEY (id);

ALTER TABLE ONLY achievement_series_text
    ADD CONSTRAINT achievement_series_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (uuid);

ALTER TABLE ONLY skills
    ADD CONSTRAINT skills_pkey PRIMARY KEY (id);

ALTER TABLE ONLY skills_text
    ADD CONSTRAINT skills_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY users_books
    ADD CONSTRAINT users_books_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY users
    ADD CONSTRAINT users_pkey PRIMARY KEY (username);

ALTER TABLE ONLY warps
    ADD CONSTRAINT warps_pkey PRIMARY KEY (id, timestamp);

ALTER TABLE ONLY warps_stats_4
    ADD CONSTRAINT warps_stats_4_pkey PRIMARY KEY (uid, gacha_type);

ALTER TABLE ONLY warps_stats_5
    ADD CONSTRAINT warps_stats_5_pkey PRIMARY KEY (uid, gacha_type);

ALTER TABLE ONLY warps_stats
    ADD CONSTRAINT warps_stats_pkey PRIMARY KEY (uid, gacha_type);

ALTER TABLE ONLY achievements_percent
    ADD CONSTRAINT achievements_percent_id_fkey FOREIGN KEY (id) REFERENCES achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY achievements
    ADD CONSTRAINT achievements_series_fkey FOREIGN KEY (series) REFERENCES achievement_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY achievements_text
    ADD CONSTRAINT achievements_text_id_fkey FOREIGN KEY (id) REFERENCES achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY admins
    ADD CONSTRAINT admins_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY book_series_text
    ADD CONSTRAINT book_series_text_id_fkey FOREIGN KEY (id) REFERENCES book_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY book_series
    ADD CONSTRAINT book_series_world_fkey FOREIGN KEY (world) REFERENCES book_series_worlds (id) ON DELETE CASCADE;

ALTER TABLE ONLY book_series_worlds_text
    ADD CONSTRAINT book_series_worlds_text_id_fkey FOREIGN KEY (id) REFERENCES book_series_worlds (id) ON DELETE CASCADE;

ALTER TABLE ONLY books_percent
    ADD CONSTRAINT books_percent_id_fkey FOREIGN KEY (id) REFERENCES books (id) ON DELETE CASCADE;

ALTER TABLE ONLY books
    ADD CONSTRAINT books_series_fkey FOREIGN KEY (series) REFERENCES book_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY books_text
    ADD CONSTRAINT books_text_id_fkey FOREIGN KEY (id) REFERENCES books (id) ON DELETE CASCADE;

ALTER TABLE ONLY characters_text
    ADD CONSTRAINT characters_text_id_fkey FOREIGN KEY (id) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY community_tier_list_entries
    ADD CONSTRAINT community_tier_list_entries_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY users_achievements
    ADD CONSTRAINT users_achievements_id_fkey FOREIGN KEY (id) REFERENCES achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY connections
    ADD CONSTRAINT connections_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY connections
    ADD CONSTRAINT connections_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON DELETE CASCADE;

ALTER TABLE ONLY light_cones_text
    ADD CONSTRAINT light_cones_text_id_fkey FOREIGN KEY (id) REFERENCES light_cones (id) ON DELETE CASCADE;

ALTER TABLE ONLY scores_achievement
    ADD CONSTRAINT scores_achievement_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY achievement_series_text
    ADD CONSTRAINT achievements_series_text_id_fkey FOREIGN KEY (id) REFERENCES achievement_series (id) ON DELETE CASCADE;

ALTER TABLE ONLY sessions
    ADD CONSTRAINT sessions_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY skills
    ADD CONSTRAINT skills_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY skills_text
    ADD CONSTRAINT skills_text_id_fkey FOREIGN KEY (id) REFERENCES skills (id) ON DELETE CASCADE;

ALTER TABLE ONLY users_achievements
    ADD CONSTRAINT users_achievements_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY users_books
    ADD CONSTRAINT users_books_id_fkey FOREIGN KEY (id) REFERENCES books (id) ON DELETE CASCADE;

ALTER TABLE ONLY users_books
    ADD CONSTRAINT users_books_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY warps
    ADD CONSTRAINT warps_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps
    ADD CONSTRAINT warps_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES light_cones (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps
    ADD CONSTRAINT warps_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

