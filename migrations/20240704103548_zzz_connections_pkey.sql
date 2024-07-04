ALTER TABLE ONLY zzz_connections
    DROP CONSTRAINT zzz_connections_pkey;

ALTER TABLE ONLY zzz_connections
    ADD CONSTRAINT zzz_connections_pkey PRIMARY KEY (uid, username);

