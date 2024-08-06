ALTER TABLE ONLY gi_wishes_beginner
    ADD CONSTRAINT gi_wishes_beginner_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_standard
    ADD CONSTRAINT gi_wishes_standard_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_character
    ADD CONSTRAINT gi_wishes_character_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_weapon
    ADD CONSTRAINT gi_wishes_weapon_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_chronicled
    ADD CONSTRAINT gi_wishes_chronicled_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_beginner
    ADD CONSTRAINT gi_wishes_beginner_character_fkey FOREIGN KEY (character) REFERENCES gi_characters (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_standard
    ADD CONSTRAINT gi_wishes_standard_character_fkey FOREIGN KEY (character) REFERENCES gi_characters (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_character
    ADD CONSTRAINT gi_wishes_character_character_fkey FOREIGN KEY (character) REFERENCES gi_characters (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_weapon
    ADD CONSTRAINT gi_wishes_weapon_character_fkey FOREIGN KEY (character) REFERENCES gi_characters (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_chronicled
    ADD CONSTRAINT gi_wishes_chronicled_character_fkey FOREIGN KEY (character) REFERENCES gi_characters (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_beginner
    ADD CONSTRAINT gi_wishes_beginner_weapon_fkey FOREIGN KEY (weapon) REFERENCES gi_weapons (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_standard
    ADD CONSTRAINT gi_wishes_standard_weapon_fkey FOREIGN KEY (weapon) REFERENCES gi_weapons (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_character
    ADD CONSTRAINT gi_wishes_character_weapon_fkey FOREIGN KEY (weapon) REFERENCES gi_weapons (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_weapon
    ADD CONSTRAINT gi_wishes_weapon_weapon_fkey FOREIGN KEY (weapon) REFERENCES gi_weapons (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_chronicled
    ADD CONSTRAINT gi_wishes_chronicled_weapon_fkey FOREIGN KEY (weapon) REFERENCES gi_weapons (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_standard
    ADD CONSTRAINT gi_wishes_stats_standard_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_character
    ADD CONSTRAINT gi_wishes_stats_character_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_weapon
    ADD CONSTRAINT gi_wishes_stats_weapon_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_chronicled
    ADD CONSTRAINT gi_wishes_stats_chronicled_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_global_standard
    ADD CONSTRAINT gi_wishes_stats_global_standard_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_global_character
    ADD CONSTRAINT gi_wishes_stats_global_character_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_global_weapon
    ADD CONSTRAINT gi_wishes_stats_global_weapon_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_wishes_stats_global_chronicled
    ADD CONSTRAINT gi_wishes_stats_global_chronicled_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles (uid) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_characters_text
    ADD CONSTRAINT gi_characters_text_character_fkey FOREIGN KEY (id) REFERENCES gi_characters (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_weapons_text
    ADD CONSTRAINT gi_weapons_text_weapon_fkey FOREIGN KEY (id) REFERENCES gi_weapons (id) ON UPDATE CASCADE ON DELETE CASCADE;

