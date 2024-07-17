CREATE TABLE zzz_signals_standard (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE zzz_signals_special (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE zzz_signals_w_engine (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE zzz_signals_bangboo (
    id bigint NOT NULL,
    uid integer NOT NULL,
    bangboo integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

ALTER TABLE ONLY zzz_signals_standard
    ADD CONSTRAINT zzz_signals_standard_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals_special
    ADD CONSTRAINT zzz_signals_special_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals_w_engine
    ADD CONSTRAINT zzz_signals_w_engine_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals_bangboo
    ADD CONSTRAINT zzz_signals_bangboo_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals_standard
    ADD CONSTRAINT zzz_signals_standard_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_standard
    ADD CONSTRAINT zzz_signals_standard_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_standard
    ADD CONSTRAINT zzz_signals_standard_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_special
    ADD CONSTRAINT zzz_signals_special_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_special
    ADD CONSTRAINT zzz_signals_special_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_special
    ADD CONSTRAINT zzz_signals_special_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_w_engine
    ADD CONSTRAINT zzz_signals_w_engine_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_w_engine
    ADD CONSTRAINT zzz_signals_w_engine_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_w_engine
    ADD CONSTRAINT zzz_signals_w_engine_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_bangboo
    ADD CONSTRAINT zzz_signals_bangboo_character_fkey FOREIGN KEY (bangboo) REFERENCES zzz_bangboos (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_bangboo
    ADD CONSTRAINT zzz_signals_bangboo_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_bangboo
    ADD CONSTRAINT zzz_signals_bangboo_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

INSERT INTO zzz_signals_standard
SELECT
    id,
    uid,
    character,
    w_engine,
    timestamp,
    official
FROM
    zzz_signals
WHERE
    gacha_type = 'standard';

INSERT INTO zzz_signals_special
SELECT
    id,
    uid,
    character,
    w_engine,
    timestamp,
    official
FROM
    zzz_signals
WHERE
    gacha_type = 'special';

INSERT INTO zzz_signals_w_engine
SELECT
    id,
    uid,
    character,
    w_engine,
    timestamp,
    official
FROM
    zzz_signals
WHERE
    gacha_type = 'w_engine';

INSERT INTO zzz_signals_bangboo
SELECT
    id,
    uid,
    bangboo,
    w_engine,
    timestamp,
    official
FROM
    zzz_signals
WHERE
    gacha_type = 'bangboo';

DROP TABLE zzz_signals;

