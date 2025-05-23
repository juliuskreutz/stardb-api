CREATE TABLE IF NOT EXISTS gi_banners (
    id integer NOT NULL,
    start timestamp with time zone NOT NULL,
    "end" timestamp with time zone NOT NULL,
    character integer,
    weapon integer
);

ALTER TABLE ONLY gi_banners
    ADD CONSTRAINT gi_banners_pkey PRIMARY KEY (id);

ALTER TABLE ONLY gi_banners
    ADD CONSTRAINT gi_banners_character_fkey FOREIGN KEY (character) REFERENCES gi_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY gi_banners
    ADD CONSTRAINT gi_banners_weapon_fkey FOREIGN KEY (weapon) REFERENCES gi_weapons (id) ON DELETE CASCADE;

