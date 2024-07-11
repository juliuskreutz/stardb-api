DROP INDEX warps_uid_index;

DROP INDEX zzz_signals_uid_index;

ALTER TABLE ONLY warps
    DROP CONSTRAINT warps_pkey;

ALTER TABLE ONLY zzz_signals
    DROP CONSTRAINT zzz_signals_pkey;

ALTER TABLE ONLY warps
    ADD CONSTRAINT warps_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals
    ADD CONSTRAINT zzz_signals_pkey PRIMARY KEY (uid, id);

