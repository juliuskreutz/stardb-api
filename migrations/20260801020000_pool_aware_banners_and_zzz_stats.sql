ALTER TABLE banners
    ADD COLUMN character_gacha_type integer,
    ADD COLUMN light_cone_gacha_type integer;

UPDATE banners SET character_gacha_type = 11 WHERE character IS NOT NULL;
UPDATE banners SET light_cone_gacha_type = 12 WHERE light_cone IS NOT NULL;

ALTER TABLE banners
    ADD CONSTRAINT banners_valid_range CHECK (start < "end"),
    ADD CONSTRAINT banners_featured_item CHECK (character IS NOT NULL OR light_cone IS NOT NULL);

ALTER TABLE gi_banners
    ADD COLUMN character_gacha_type integer,
    ADD COLUMN weapon_gacha_type integer;

UPDATE gi_banners SET character_gacha_type = 301 WHERE character IS NOT NULL;
UPDATE gi_banners SET weapon_gacha_type = 302 WHERE weapon IS NOT NULL;

-- After deployment, operators should review possible special-pool backfills with:
-- SELECT id, name, start, "end", character, weapon, character_gacha_type, weapon_gacha_type
-- FROM gi_banners WHERE lower(name) LIKE '%chronicled%';
-- SELECT id, name, start, "end", character, light_cone, character_gacha_type, light_cone_gacha_type
-- FROM banners WHERE lower(name) LIKE '%collab%' OR lower(name) LIKE '%fate%';
-- Correct only confirmed rows through the existing admin banner PUT routes.

ALTER TABLE gi_banners
    ADD CONSTRAINT gi_banners_valid_range CHECK (start < "end"),
    ADD CONSTRAINT gi_banners_featured_item CHECK (character IS NOT NULL OR weapon IS NOT NULL);

CREATE TABLE zzz_banners (
    id integer PRIMARY KEY,
    name text NOT NULL DEFAULT '',
    start timestamp with time zone NOT NULL,
    "end" timestamp with time zone NOT NULL,
    character integer REFERENCES zzz_characters(id) ON DELETE CASCADE,
    character_gacha_type integer,
    w_engine integer REFERENCES zzz_w_engines(id) ON DELETE CASCADE,
    w_engine_gacha_type integer,
    bangboo integer REFERENCES zzz_bangboos(id) ON DELETE CASCADE,
    bangboo_gacha_type integer,
    CONSTRAINT zzz_banners_valid_range CHECK (start < "end"),
    CONSTRAINT zzz_banners_featured_item CHECK (character IS NOT NULL OR w_engine IS NOT NULL OR bangboo IS NOT NULL)
);

CREATE TABLE zzz_signals_stats_exclusive_rescreening (
    uid integer PRIMARY KEY REFERENCES zzz_uids(uid) ON DELETE CASCADE,
    luck_a double precision NOT NULL,
    luck_s double precision NOT NULL,
    win_rate double precision NOT NULL,
    win_streak integer NOT NULL,
    loss_streak integer NOT NULL
);

CREATE TABLE zzz_signals_stats_w_engine_reverberation (
    LIKE zzz_signals_stats_exclusive_rescreening INCLUDING ALL
);
ALTER TABLE zzz_signals_stats_w_engine_reverberation
    ADD CONSTRAINT zzz_signals_stats_w_engine_reverberation_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids(uid) ON DELETE CASCADE;

CREATE TABLE zzz_signals_stats_global_exclusive_rescreening (
    uid integer PRIMARY KEY REFERENCES zzz_uids(uid) ON DELETE CASCADE,
    count_percentile double precision NOT NULL,
    luck_a_percentile double precision NOT NULL,
    luck_s_percentile double precision NOT NULL
);
CREATE TABLE zzz_signals_stats_global_w_engine_reverberation (
    LIKE zzz_signals_stats_global_exclusive_rescreening INCLUDING ALL
);
ALTER TABLE zzz_signals_stats_global_w_engine_reverberation
    ADD CONSTRAINT zzz_signals_stats_global_w_engine_reverberation_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids(uid) ON DELETE CASCADE;
