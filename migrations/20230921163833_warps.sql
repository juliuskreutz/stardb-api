DROP TABLE warp_standard_characters;
DROP TABLE warp_standard_light_cones;
DROP TABLE warp_departure_characters;
DROP TABLE warp_departure_light_cones;
DROP TABLE warp_special_characters;
DROP TABLE warp_special_light_cones;
DROP TABLE warp_lc_characters;
DROP TABLE warp_lc_light_cones;

CREATE TABLE IF NOT EXISTS warps (
    id INT8 NOT NULL PRIMARY KEY,
    uid INT8 NOT NULL REFERENCES mihomo ON DELETE CASCADE,
    gacha_type TEXT NOT NULL,
    character INT4 REFERENCES characters ON DELETE CASCADE,
    light_cone INT4 REFERENCES light_cones ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL
);