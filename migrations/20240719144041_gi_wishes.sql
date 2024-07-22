CREATE TABLE IF NOT EXISTS gi_characters (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_characters_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_connections (
    uid integer NOT NULL,
    username text NOT NULL,
    verified boolean NOT NULL,
    private boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_weapons (
    id integer NOT NULL,
    rarity integer DEFAULT 0 NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_weapons_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_profiles (
    uid integer NOT NULL,
    name text NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_beginner (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    weapon integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_standard (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    weapon integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_character (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    weapon integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_weapon (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    weapon integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS gi_wishes_chronicled (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    weapon integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

ALTER TABLE ONLY gi_characters
    ADD CONSTRAINT gi_characters_pkey PRIMARY KEY (id);

ALTER TABLE ONLY gi_characters_text
    ADD CONSTRAINT gi_characters_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY gi_connections
    ADD CONSTRAINT gi_connections_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_weapons
    ADD CONSTRAINT gi_weapons_pkey PRIMARY KEY (id);

ALTER TABLE ONLY gi_weapons_text
    ADD CONSTRAINT gi_weapons_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY gi_profiles
    ADD CONSTRAINT gi_profiles_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_beginner
    ADD CONSTRAINT gi_wishes_beginner_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY gi_wishes_standard
    ADD CONSTRAINT gi_wishes_standard_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY gi_wishes_character
    ADD CONSTRAINT gi_wishes_character_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY gi_wishes_weapon
    ADD CONSTRAINT gi_wishes_weapon_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY gi_wishes_chronicled
    ADD CONSTRAINT gi_wishes_chronicled_pkey PRIMARY KEY (uid, id);

