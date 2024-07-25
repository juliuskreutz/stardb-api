ALTER TABLE ONLY gi_connections
    DROP CONSTRAINT gi_connections_pkey;

ALTER TABLE ONLY gi_connections
    ADD CONSTRAINT gi_connections_pkey PRIMARY KEY (uid, username);

ALTER TABLE ONLY gi_connections
    ADD CONSTRAINT gi_connections_uid_fkey FOREIGN KEY (uid) REFERENCES gi_profiles ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY gi_connections
    ADD CONSTRAINT gi_connections_username_fkey FOREIGN KEY (username) REFERENCES users ON UPDATE CASCADE ON DELETE CASCADE;

