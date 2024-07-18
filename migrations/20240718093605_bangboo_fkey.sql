ALTER TABLE ONLY zzz_bangboos
    DROP CONSTRAINT zzz_bangboos_text_id_fkey;

ALTER TABLE ONLY zzz_bangboos_text
    ADD CONSTRAINT zzz_bangboos_text_id_fkey FOREIGN KEY (id) REFERENCES zzz_bangboos (id) ON DELETE CASCADE;

