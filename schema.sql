--
-- PostgreSQL database dump
--
-- Dumped from database version 14.12 (Ubuntu 14.12-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.12 (Ubuntu 14.12-0ubuntu0.22.04.1)
SET statement_timeout = 0;

SET lock_timeout = 0;

SET idle_in_transaction_session_timeout = 0;

SET client_encoding = 'UTF8';

SET standard_conforming_strings = ON;

SELECT
    pg_catalog.set_config('search_path', '', FALSE);

SET check_function_bodies = FALSE;

SET xmloption = content;

SET client_min_messages = warning;

SET row_security = OFF;

--
-- Name: fuzzystrmatch; Type: EXTENSION; Schema: -; Owner: -
--
CREATE EXTENSION IF NOT EXISTS fuzzystrmatch WITH SCHEMA public;

--
-- Name: EXTENSION fuzzystrmatch; Type: COMMENT; Schema: -; Owner: -
--
COMMENT ON EXTENSION fuzzystrmatch IS 'determine similarities and distance between strings';

--
-- Name: gacha_type; Type: TYPE; Schema: public; Owner: -
--
CREATE TYPE public.gacha_type AS ENUM (
    'departure',
    'standard',
    'special',
    'lc'
);

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);

--
-- Name: achievement_series; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.achievement_series (
    id integer NOT NULL,
    priority integer NOT NULL
);

--
-- Name: achievement_series_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.achievement_series_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

--
-- Name: achievements; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.achievements (
    id integer NOT NULL,
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

--
-- Name: achievements_percent; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.achievements_percent (
    id integer NOT NULL,
    percent double precision NOT NULL
);

--
-- Name: achievements_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.achievements_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    description text NOT NULL
);

--
-- Name: admins; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.admins (
    username text NOT NULL
);

--
-- Name: book_series; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.book_series (
    id integer NOT NULL,
    world integer NOT NULL,
    bookshelf boolean DEFAULT FALSE NOT NULL
);

--
-- Name: book_series_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.book_series_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

--
-- Name: book_series_worlds; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.book_series_worlds (
    id integer NOT NULL
);

--
-- Name: book_series_worlds_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.book_series_worlds_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

--
-- Name: books; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.books (
    id integer NOT NULL,
    series_inside integer NOT NULL,
    series integer NOT NULL,
    comment text,
    image1 text,
    image2 text,
    icon integer
);

--
-- Name: books_percent; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.books_percent (
    id integer NOT NULL,
    percent double precision NOT NULL
);

--
-- Name: books_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.books_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

--
-- Name: characters; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.characters (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

--
-- Name: characters_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.characters_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    element text NOT NULL,
    path text NOT NULL
);

--
-- Name: community_tier_list_entries; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.community_tier_list_entries (
    "character" integer NOT NULL,
    eidolon integer NOT NULL,
    average double precision NOT NULL,
    variance double precision NOT NULL,
    votes integer NOT NULL,
    total_votes integer DEFAULT 0 NOT NULL,
    quartile_1 double precision DEFAULT 0 NOT NULL,
    quartile_3 double precision DEFAULT 0 NOT NULL,
    confidence_interval_95 double precision DEFAULT 0 NOT NULL
);

--
-- Name: community_tier_list_sextiles; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.community_tier_list_sextiles (
    value double precision NOT NULL,
    id integer NOT NULL
);

--
-- Name: connections; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.connections (
    uid integer NOT NULL,
    username text NOT NULL,
    verified boolean DEFAULT FALSE NOT NULL,
    private boolean DEFAULT FALSE NOT NULL
);

--
-- Name: light_cones; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.light_cones (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

--
-- Name: light_cones_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.light_cones_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL,
    path text DEFAULT ''::text NOT NULL
);

--
-- Name: mihomo; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.mihomo (
    uid integer NOT NULL,
    region text NOT NULL,
    name text NOT NULL,
    level integer NOT NULL,
    signature text NOT NULL,
    avatar_icon text NOT NULL,
    achievement_count integer NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

--
-- Name: scores_achievement; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.scores_achievement (
    uid integer NOT NULL,
    "timestamp" timestamp with time zone NOT NULL
);

--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.sessions (
    uuid uuid NOT NULL,
    username text NOT NULL,
    expiry timestamp with time zone NOT NULL
);

--
-- Name: skills; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.skills (
    id integer NOT NULL,
    "character" integer NOT NULL
);

--
-- Name: skills_text; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.skills_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.users (
    username text NOT NULL,
    password text NOT NULL,
    email text
);

--
-- Name: users_achievements_completed; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.users_achievements_completed (
    username text NOT NULL,
    id integer NOT NULL
);

--
-- Name: users_achievements_favorites; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.users_achievements_favorites (
    username text NOT NULL,
    id integer NOT NULL
);

--
-- Name: users_books_completed; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.users_books_completed (
    username text NOT NULL,
    id integer NOT NULL
);

--
-- Name: users_books_favorites; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.users_books_favorites (
    username text NOT NULL,
    id integer NOT NULL
);

--
-- Name: warps; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.warps (
    id bigint NOT NULL,
    uid integer NOT NULL,
    gacha_type text NOT NULL,
    "character" integer,
    light_cone integer,
    "timestamp" timestamp with time zone NOT NULL,
    official boolean DEFAULT FALSE NOT NULL
);

--
-- Name: warps_stats; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.warps_stats (
    uid integer NOT NULL,
    gacha_type public.gacha_type NOT NULL,
    count integer NOT NULL,
    rank integer NOT NULL
);

--
-- Name: warps_stats_4; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.warps_stats_4 (
    uid integer NOT NULL,
    gacha_type public.gacha_type NOT NULL,
    count integer NOT NULL,
    avg double precision NOT NULL,
    rank_count integer NOT NULL,
    rank_avg integer NOT NULL,
    median integer DEFAULT 0 NOT NULL,
    rank_median integer DEFAULT 0 NOT NULL
);

--
-- Name: warps_stats_5; Type: TABLE; Schema: public; Owner: -
--
CREATE TABLE public.warps_stats_5 (
    uid integer NOT NULL,
    gacha_type public.gacha_type NOT NULL,
    count integer NOT NULL,
    avg double precision NOT NULL,
    rank_count integer NOT NULL,
    rank_avg integer NOT NULL,
    median integer DEFAULT 0 NOT NULL,
    rank_median integer DEFAULT 0 NOT NULL
);

--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);

--
-- Name: achievement_series achievement_series_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievement_series
    ADD CONSTRAINT achievement_series_pkey PRIMARY KEY (id);

--
-- Name: achievement_series_text achievement_series_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievement_series_text
    ADD CONSTRAINT achievement_series_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: achievements_percent achievements_percent_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievements_percent
    ADD CONSTRAINT achievements_percent_pkey PRIMARY KEY (id);

--
-- Name: achievements achievements_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievements
    ADD CONSTRAINT achievements_pkey PRIMARY KEY (id);

--
-- Name: achievements_text achievements_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievements_text
    ADD CONSTRAINT achievements_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: admins admins_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.admins
    ADD CONSTRAINT admins_pkey PRIMARY KEY (username);

--
-- Name: book_series book_series_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series
    ADD CONSTRAINT book_series_pkey PRIMARY KEY (id);

--
-- Name: book_series_text book_series_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series_text
    ADD CONSTRAINT book_series_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: book_series_worlds book_series_worlds_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series_worlds
    ADD CONSTRAINT book_series_worlds_pkey PRIMARY KEY (id);

--
-- Name: book_series_worlds_text book_series_worlds_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series_worlds_text
    ADD CONSTRAINT book_series_worlds_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: books_percent books_percent_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.books_percent
    ADD CONSTRAINT books_percent_pkey PRIMARY KEY (id);

--
-- Name: books books_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.books
    ADD CONSTRAINT books_pkey PRIMARY KEY (id);

--
-- Name: books_text books_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.books_text
    ADD CONSTRAINT books_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: characters characters_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.characters
    ADD CONSTRAINT characters_pkey PRIMARY KEY (id);

--
-- Name: characters_text characters_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.characters_text
    ADD CONSTRAINT characters_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: community_tier_list_entries community_tier_list_entries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.community_tier_list_entries
    ADD CONSTRAINT community_tier_list_entries_pkey PRIMARY KEY ("character", eidolon);

--
-- Name: community_tier_list_sextiles community_tier_list_sextiles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.community_tier_list_sextiles
    ADD CONSTRAINT community_tier_list_sextiles_pkey PRIMARY KEY (id);

--
-- Name: connections connections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.connections
    ADD CONSTRAINT connections_pkey PRIMARY KEY (uid, username);

--
-- Name: light_cones light_cones_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.light_cones
    ADD CONSTRAINT light_cones_pkey PRIMARY KEY (id);

--
-- Name: light_cones_text light_cones_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.light_cones_text
    ADD CONSTRAINT light_cones_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: mihomo mihomo_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.mihomo
    ADD CONSTRAINT mihomo_pkey PRIMARY KEY (uid);

--
-- Name: scores_achievement scores_achievement_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.scores_achievement
    ADD CONSTRAINT scores_achievement_pkey PRIMARY KEY (uid);

--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_pkey PRIMARY KEY (uuid);

--
-- Name: skills skills_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.skills
    ADD CONSTRAINT skills_pkey PRIMARY KEY (id);

--
-- Name: skills_text skills_text_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.skills_text
    ADD CONSTRAINT skills_text_pkey PRIMARY KEY (id, LANGUAGE);

--
-- Name: users_achievements_completed users_achievements_completed_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_achievements_completed
    ADD CONSTRAINT users_achievements_completed_pkey PRIMARY KEY (username, id);

--
-- Name: users_achievements_favorites users_achievements_favorites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_achievements_favorites
    ADD CONSTRAINT users_achievements_favorites_pkey PRIMARY KEY (username, id);

--
-- Name: users_books_completed users_books_completed_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_books_completed
    ADD CONSTRAINT users_books_completed_pkey PRIMARY KEY (username, id);

--
-- Name: users_books_favorites users_books_favorites_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_books_favorites
    ADD CONSTRAINT users_books_favorites_pkey PRIMARY KEY (username, id);

--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (username);

--
-- Name: warps warps_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps
    ADD CONSTRAINT warps_pkey PRIMARY KEY (id, "timestamp");

--
-- Name: warps_stats_4 warps_stats_4_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps_stats_4
    ADD CONSTRAINT warps_stats_4_pkey PRIMARY KEY (uid, gacha_type);

--
-- Name: warps_stats_5 warps_stats_5_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps_stats_5
    ADD CONSTRAINT warps_stats_5_pkey PRIMARY KEY (uid, gacha_type);

--
-- Name: warps_stats warps_stats_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps_stats
    ADD CONSTRAINT warps_stats_pkey PRIMARY KEY (uid, gacha_type);

--
-- Name: warps_uid_index; Type: INDEX; Schema: public; Owner: -
--
CREATE INDEX warps_uid_index ON public.warps USING btree (uid);

--
-- Name: achievements_percent achievements_percent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievements_percent
    ADD CONSTRAINT achievements_percent_id_fkey FOREIGN KEY (id) REFERENCES public.achievements (id) ON DELETE CASCADE;

--
-- Name: achievements achievements_series_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievements
    ADD CONSTRAINT achievements_series_fkey FOREIGN KEY (series) REFERENCES public.achievement_series (id) ON DELETE CASCADE;

--
-- Name: achievement_series_text achievements_series_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievement_series_text
    ADD CONSTRAINT achievements_series_text_id_fkey FOREIGN KEY (id) REFERENCES public.achievement_series (id) ON DELETE CASCADE;

--
-- Name: achievements_text achievements_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.achievements_text
    ADD CONSTRAINT achievements_text_id_fkey FOREIGN KEY (id) REFERENCES public.achievements (id) ON DELETE CASCADE;

--
-- Name: admins admins_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.admins
    ADD CONSTRAINT admins_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- Name: book_series_text book_series_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series_text
    ADD CONSTRAINT book_series_text_id_fkey FOREIGN KEY (id) REFERENCES public.book_series (id) ON DELETE CASCADE;

--
-- Name: book_series book_series_world_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series
    ADD CONSTRAINT book_series_world_fkey FOREIGN KEY (world) REFERENCES public.book_series_worlds (id) ON DELETE CASCADE;

--
-- Name: book_series_worlds_text book_series_worlds_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.book_series_worlds_text
    ADD CONSTRAINT book_series_worlds_text_id_fkey FOREIGN KEY (id) REFERENCES public.book_series_worlds (id) ON DELETE CASCADE;

--
-- Name: books_percent books_percent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.books_percent
    ADD CONSTRAINT books_percent_id_fkey FOREIGN KEY (id) REFERENCES public.books (id) ON DELETE CASCADE;

--
-- Name: books books_series_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.books
    ADD CONSTRAINT books_series_fkey FOREIGN KEY (series) REFERENCES public.book_series (id) ON DELETE CASCADE;

--
-- Name: books_text books_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.books_text
    ADD CONSTRAINT books_text_id_fkey FOREIGN KEY (id) REFERENCES public.books (id) ON DELETE CASCADE;

--
-- Name: characters_text characters_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.characters_text
    ADD CONSTRAINT characters_text_id_fkey FOREIGN KEY (id) REFERENCES public.characters (id) ON DELETE CASCADE;

--
-- Name: community_tier_list_entries community_tier_list_entries_character_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.community_tier_list_entries
    ADD CONSTRAINT community_tier_list_entries_character_fkey FOREIGN KEY ("character") REFERENCES public.characters (id) ON DELETE CASCADE;

--
-- Name: connections connections_uid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.connections
    ADD CONSTRAINT connections_uid_fkey FOREIGN KEY (uid) REFERENCES public.mihomo (uid) ON DELETE CASCADE;

--
-- Name: connections connections_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.connections
    ADD CONSTRAINT connections_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON DELETE CASCADE;

--
-- Name: light_cones_text light_cones_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.light_cones_text
    ADD CONSTRAINT light_cones_text_id_fkey FOREIGN KEY (id) REFERENCES public.light_cones (id) ON DELETE CASCADE;

--
-- Name: scores_achievement scores_achievement_uid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.scores_achievement
    ADD CONSTRAINT scores_achievement_uid_fkey FOREIGN KEY (uid) REFERENCES public.mihomo (uid) ON DELETE CASCADE;

--
-- Name: sessions sessions_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.sessions
    ADD CONSTRAINT sessions_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- Name: skills skills_character_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.skills
    ADD CONSTRAINT skills_character_fkey FOREIGN KEY ("character") REFERENCES public.characters (id) ON DELETE CASCADE;

--
-- Name: skills_text skills_text_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.skills_text
    ADD CONSTRAINT skills_text_id_fkey FOREIGN KEY (id) REFERENCES public.skills (id) ON DELETE CASCADE;

--
-- Name: users_achievements_completed users_achievements_completed_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_achievements_completed
    ADD CONSTRAINT users_achievements_completed_id_fkey FOREIGN KEY (id) REFERENCES public.achievements (id) ON DELETE CASCADE;

--
-- Name: users_achievements_completed users_achievements_completed_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_achievements_completed
    ADD CONSTRAINT users_achievements_completed_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- Name: users_achievements_favorites users_achievements_favorites_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_achievements_favorites
    ADD CONSTRAINT users_achievements_favorites_id_fkey FOREIGN KEY (id) REFERENCES public.achievements (id) ON DELETE CASCADE;

--
-- Name: users_achievements_favorites users_achievements_favorites_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_achievements_favorites
    ADD CONSTRAINT users_achievements_favorites_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- Name: users_books_completed users_books_completed_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_books_completed
    ADD CONSTRAINT users_books_completed_id_fkey FOREIGN KEY (id) REFERENCES public.books (id) ON DELETE CASCADE;

--
-- Name: users_books_completed users_books_completed_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_books_completed
    ADD CONSTRAINT users_books_completed_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- Name: users_books_favorites users_books_favorites_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_books_favorites
    ADD CONSTRAINT users_books_favorites_id_fkey FOREIGN KEY (id) REFERENCES public.books (id) ON DELETE CASCADE;

--
-- Name: users_books_favorites users_books_favorites_username_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.users_books_favorites
    ADD CONSTRAINT users_books_favorites_username_fkey FOREIGN KEY (username) REFERENCES public.users (username) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- Name: warps warps_character_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps
    ADD CONSTRAINT warps_character_fkey FOREIGN KEY ("character") REFERENCES public.characters (id) ON DELETE CASCADE;

--
-- Name: warps warps_light_cone_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps
    ADD CONSTRAINT warps_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES public.light_cones (id) ON DELETE CASCADE;

--
-- Name: warps_stats_4 warps_stats_4_uid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps_stats_4
    ADD CONSTRAINT warps_stats_4_uid_fkey FOREIGN KEY (uid) REFERENCES public.mihomo (uid) ON DELETE CASCADE;

--
-- Name: warps_stats_5 warps_stats_5_uid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps_stats_5
    ADD CONSTRAINT warps_stats_5_uid_fkey FOREIGN KEY (uid) REFERENCES public.mihomo (uid) ON DELETE CASCADE;

--
-- Name: warps_stats warps_stats_uid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps_stats
    ADD CONSTRAINT warps_stats_uid_fkey FOREIGN KEY (uid) REFERENCES public.mihomo (uid) ON DELETE CASCADE;

--
-- Name: warps warps_uid_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--
ALTER TABLE ONLY public.warps
    ADD CONSTRAINT warps_uid_fkey FOREIGN KEY (uid) REFERENCES public.mihomo (uid) ON DELETE CASCADE;

--
-- PostgreSQL database dump complete
--
