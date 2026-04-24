CREATE TABLE IF NOT EXISTS zzz_banners (
    id integer NOT NULL,
    name text NOT NULL DEFAULT '',
    start timestamp with time zone NOT NULL,
    "end" timestamp with time zone NOT NULL,
    character integer,
    w_engine integer,
    bangboo integer
);

ALTER TABLE ONLY zzz_banners
    ADD CONSTRAINT zzz_banners_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_banners
    ADD CONSTRAINT zzz_banners_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_banners
    ADD CONSTRAINT zzz_banners_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_banners
    ADD CONSTRAINT zzz_banners_bangboo_fkey FOREIGN KEY (bangboo) REFERENCES zzz_bangboos (id) ON DELETE CASCADE;
