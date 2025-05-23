CREATE TABLE IF NOT EXISTS banners (
    id integer NOT NULL,
    start timestamp with time zone NOT NULL,
    "end" timestamp with time zone NOT NULL,
    character integer,
    light_cone integer
);

ALTER TABLE ONLY banners
    ADD CONSTRAINT banners_pkey PRIMARY KEY (id);

ALTER TABLE ONLY banners
    ADD CONSTRAINT banners_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY banners
    ADD CONSTRAINT banners_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES light_cones (id) ON DELETE CASCADE;

