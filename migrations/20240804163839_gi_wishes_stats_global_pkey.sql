ALTER TABLE ONLY gi_wishes_stats_global_standard
    ADD CONSTRAINT gi_wishes_stats_global_standard_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_stats_global_character
    ADD CONSTRAINT gi_wishes_stats_global_character_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_stats_global_weapon
    ADD CONSTRAINT gi_wishes_stats_global_weapon_pkey PRIMARY KEY (uid);

ALTER TABLE ONLY gi_wishes_stats_global_chronicled
    ADD CONSTRAINT gi_wishes_stats_global_chronicled_pkey PRIMARY KEY (uid);

