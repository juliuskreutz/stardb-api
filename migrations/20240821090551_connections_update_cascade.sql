ALTER TABLE ONLY connections
    DROP CONSTRAINT connections_username_fkey;

ALTER TABLE ONLY connections
    ADD CONSTRAINT connections_username_fkey FOREIGN KEY (username) REFERENCES users ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY zzz_connections
    DROP CONSTRAINT zzz_connections_username_fkey;

ALTER TABLE ONLY zzz_connections
    ADD CONSTRAINT zzz_connections_username_fkey FOREIGN KEY (username) REFERENCES users ON UPDATE CASCADE ON DELETE CASCADE;

