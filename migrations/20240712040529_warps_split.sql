CREATE TABLE warps_departure (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    light_cone integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE warps_standard (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    light_cone integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE warps_special (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    light_cone integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE warps_lc (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    light_cone integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

ALTER TABLE ONLY warps_departure
    ADD CONSTRAINT warps_departure_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY warps_standard
    ADD CONSTRAINT warps_standard_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY warps_special
    ADD CONSTRAINT warps_special_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY warps_lc
    ADD CONSTRAINT warps_lc_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY warps_departure
    ADD CONSTRAINT warps_departure_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_departure
    ADD CONSTRAINT warps_departure_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES light_cones (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_departure
    ADD CONSTRAINT warps_departure_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_standard
    ADD CONSTRAINT warps_standard_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_standard
    ADD CONSTRAINT warps_standard_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES light_cones (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_standard
    ADD CONSTRAINT warps_standard_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_special
    ADD CONSTRAINT warps_special_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_special
    ADD CONSTRAINT warps_special_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES light_cones (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_special
    ADD CONSTRAINT warps_special_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_lc
    ADD CONSTRAINT warps_lc_character_fkey FOREIGN KEY (character) REFERENCES characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_lc
    ADD CONSTRAINT warps_lc_light_cone_fkey FOREIGN KEY (light_cone) REFERENCES light_cones (id) ON DELETE CASCADE;

ALTER TABLE ONLY warps_lc
    ADD CONSTRAINT warps_lc_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

INSERT INTO warps_departure
SELECT
    id,
    uid,
    character,
    light_cone,
    timestamp,
    official
FROM
    warps
WHERE
    gacha_type = 'departure';

INSERT INTO warps_standard
SELECT
    id,
    uid,
    character,
    light_cone,
    timestamp,
    official
FROM
    warps
WHERE
    gacha_type = 'standard';

INSERT INTO warps_special
SELECT
    id,
    uid,
    character,
    light_cone,
    timestamp,
    official
FROM
    warps
WHERE
    gacha_type = 'special';

INSERT INTO warps_lc
SELECT
    id,
    uid,
    character,
    light_cone,
    timestamp,
    official
FROM
    warps
WHERE
    gacha_type = 'lc';

DROP TABLE warps;

