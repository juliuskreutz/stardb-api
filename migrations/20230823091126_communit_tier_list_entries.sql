ALTER TABLE community_tier_list_entries ADD COLUMN quartile_1 FLOAT8 NOT NULL DEFAULT 0;
ALTER TABLE community_tier_list_entries ADD COLUMN quartile_3 FLOAT8 NOT NULL DEFAULT 0;
ALTER TABLE community_tier_list_entries ADD COLUMN confidence_interval_95 FLOAT8 NOT NULL DEFAULT 0;
