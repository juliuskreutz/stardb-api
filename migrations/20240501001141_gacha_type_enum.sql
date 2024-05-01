CREATE TYPE gacha_type AS ENUM (
    'departure',
    'standard',
    'special',
    'lc'
);

ALTER TABLE warps
    ALTER COLUMN gacha_type TYPE gacha_type
    USING gacha_type::gacha_type;

ALTER TABLE warps_stats
    ALTER COLUMN gacha_type TYPE gacha_type
    USING gacha_type::gacha_type;

ALTER TABLE warps_stats_4
    ALTER COLUMN gacha_type TYPE gacha_type
    USING gacha_type::gacha_type;

ALTER TABLE warps_stats_5
    ALTER COLUMN gacha_type TYPE gacha_type
    USING gacha_type::gacha_type;

