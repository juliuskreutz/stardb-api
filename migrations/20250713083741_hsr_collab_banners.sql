CREATE TABLE IF NOT EXISTS warps_collab_lc
(
    id         BIGINT NOT NULL,
    uid        INTEGER NOT NULL,
    character  INTEGER,
    light_cone INTEGER,
    timestamp  TIMESTAMP WITH TIME ZONE NOT NULL,
    official   BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS warps_collab
(
    id         BIGINT NOT NULL,
    uid        INTEGER NOT NULL,
    character  INTEGER,
    light_cone INTEGER,
    timestamp  TIMESTAMP WITH TIME ZONE NOT NULL,
    official   BOOLEAN NOT NULL
);

ALTER TABLE warps_collab ADD CONSTRAINT warps_collab_pkey PRIMARY KEY (uid, id);

ALTER TABLE warps_collab ADD CONSTRAINT warps_collab_character_fkey
    FOREIGN KEY (character) REFERENCES characters ON DELETE CASCADE;

ALTER TABLE warps_collab ADD CONSTRAINT warps_collab_light_cone_fkey
    FOREIGN KEY (light_cone) REFERENCES light_cones ON DELETE CASCADE;

ALTER TABLE warps_collab ADD CONSTRAINT warps_collab_uid_fkey
    FOREIGN KEY (uid) REFERENCES mihomo ON DELETE CASCADE;

ALTER TABLE warps_collab_lc ADD CONSTRAINT warps_collab_lc_pkey PRIMARY KEY (uid, id);

ALTER TABLE warps_collab_lc ADD CONSTRAINT warps_collab_lc_character_fkey
    FOREIGN KEY (character) REFERENCES characters ON DELETE CASCADE;

ALTER TABLE warps_collab_lc ADD CONSTRAINT warps_collab_lc_light_cone_fkey
    FOREIGN KEY (light_cone) REFERENCES light_cones ON DELETE CASCADE;

ALTER TABLE warps_collab_lc ADD CONSTRAINT warps_collab_lc_uid_fkey
    FOREIGN KEY (uid) REFERENCES mihomo ON DELETE CASCADE;

