UPDATE warps_stats_standard
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END;

UPDATE warps_stats_special
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END,
    win_rate = CASE WHEN win_rate = 'NaN'::double precision THEN 0.0 ELSE win_rate END;

UPDATE warps_stats_lc
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END,
    win_rate = CASE WHEN win_rate = 'NaN'::double precision THEN 0.0 ELSE win_rate END;

UPDATE warps_stats_collab
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END,
    win_rate = CASE WHEN win_rate = 'NaN'::double precision THEN 0.0 ELSE win_rate END;

UPDATE warps_stats_collab_lc
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END,
    win_rate = CASE WHEN win_rate = 'NaN'::double precision THEN 0.0 ELSE win_rate END;

UPDATE gi_wishes_stats_standard
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END;

UPDATE gi_wishes_stats_character
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END,
    win_rate = CASE WHEN win_rate = 'NaN'::double precision THEN 0.0 ELSE win_rate END;

UPDATE gi_wishes_stats_weapon
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END,
    win_rate = CASE WHEN win_rate = 'NaN'::double precision THEN 0.0 ELSE win_rate END;

UPDATE gi_wishes_stats_chronicled
SET luck_4 = CASE WHEN luck_4 = 'NaN'::double precision THEN 0.0 ELSE luck_4 END,
    luck_5 = CASE WHEN luck_5 = 'NaN'::double precision THEN 0.0 ELSE luck_5 END;

ALTER TABLE warps_stats_standard
    ADD CONSTRAINT warps_stats_standard_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_standard_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision);

ALTER TABLE warps_stats_special
    ADD CONSTRAINT warps_stats_special_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_special_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_special_win_rate_range CHECK (win_rate >= 0.0 AND win_rate <= 1.0);

ALTER TABLE warps_stats_lc
    ADD CONSTRAINT warps_stats_lc_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_lc_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_lc_win_rate_range CHECK (win_rate >= 0.0 AND win_rate <= 1.0);

ALTER TABLE warps_stats_collab
    ADD CONSTRAINT warps_stats_collab_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_collab_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_collab_win_rate_range CHECK (win_rate >= 0.0 AND win_rate <= 1.0);

ALTER TABLE warps_stats_collab_lc
    ADD CONSTRAINT warps_stats_collab_lc_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_collab_lc_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision),
    ADD CONSTRAINT warps_stats_collab_lc_win_rate_range CHECK (win_rate >= 0.0 AND win_rate <= 1.0);

ALTER TABLE gi_wishes_stats_standard
    ADD CONSTRAINT gi_wishes_stats_standard_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT gi_wishes_stats_standard_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision);

ALTER TABLE gi_wishes_stats_character
    ADD CONSTRAINT gi_wishes_stats_character_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT gi_wishes_stats_character_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision),
    ADD CONSTRAINT gi_wishes_stats_character_win_rate_range CHECK (win_rate >= 0.0 AND win_rate <= 1.0);

ALTER TABLE gi_wishes_stats_weapon
    ADD CONSTRAINT gi_wishes_stats_weapon_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT gi_wishes_stats_weapon_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision),
    ADD CONSTRAINT gi_wishes_stats_weapon_win_rate_range CHECK (win_rate >= 0.0 AND win_rate <= 1.0);

ALTER TABLE gi_wishes_stats_chronicled
    ADD CONSTRAINT gi_wishes_stats_chronicled_luck_4_finite CHECK (luck_4 > '-Infinity'::double precision AND luck_4 < 'Infinity'::double precision),
    ADD CONSTRAINT gi_wishes_stats_chronicled_luck_5_finite CHECK (luck_5 > '-Infinity'::double precision AND luck_5 < 'Infinity'::double precision);
