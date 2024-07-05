ALTER TABLE ONLY zzz_signals RENAME CONSTRAINT zzz_signals_light_cone_fkey TO zzz_signals_w_engine_fkey;

CREATE TABLE IF NOT EXISTS zzz_bangboos (
    id integer NOT NULL,
    rarity integer NOT NULL
);

CREATE TABLE IF NOT EXISTS zzz_bangboos_text (
    id integer NOT NULL,
    language text
    NOT NULL,
    name text NOT NULL
);

ALTER TABLE ONLY zzz_bangboos
    ADD CONSTRAINT zzz_bangboos_pkey PRIMARY KEY (id);

ALTER TABLE ONLY zzz_bangboos_text
    ADD CONSTRAINT zzz_bangboos_text_pkey PRIMARY KEY (id, LANGUAGE);

ALTER TABLE ONLY zzz_bangboos
    ADD CONSTRAINT zzz_bangboos_text_id_fkey FOREIGN KEY (id) REFERENCES zzz_bangboos (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals
    ADD COLUMN bangboo integer;

ALTER TABLE ONLY zzz_signals
    ADD CONSTRAINT zzz_signals_bangboo_fkey FOREIGN KEY (bangboo) REFERENCES zzz_bangboos (id) ON DELETE CASCADE;

