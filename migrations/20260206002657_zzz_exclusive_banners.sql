CREATE TABLE zzz_signals_exclusive_rescreening (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

CREATE TABLE zzz_signals_w_engine_reverberation (
    id bigint NOT NULL,
    uid integer NOT NULL,
    character integer,
    w_engine integer,
    timestamp timestamp with time zone NOT NULL,
    official boolean NOT NULL
);

ALTER TABLE ONLY zzz_signals_exclusive_rescreening
    ADD CONSTRAINT zzz_signals_exclusive_rescreening_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals_w_engine_reverberation
    ADD CONSTRAINT zzz_signals_w_engine_reverberation_pkey PRIMARY KEY (uid, id);

ALTER TABLE ONLY zzz_signals_exclusive_rescreening
    ADD CONSTRAINT zzz_signals_exclusive_rescreening_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_exclusive_rescreening
    ADD CONSTRAINT zzz_signals_exclusive_rescreening_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_exclusive_rescreening
    ADD CONSTRAINT zzz_signals_exclusive_rescreening_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_w_engine_reverberation
    ADD CONSTRAINT zzz_signals_w_engine_reverberation_character_fkey FOREIGN KEY (character) REFERENCES zzz_characters (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_w_engine_reverberation
    ADD CONSTRAINT zzz_signals_w_engine_reverberation_w_engine_fkey FOREIGN KEY (w_engine) REFERENCES zzz_w_engines (id) ON DELETE CASCADE;

ALTER TABLE ONLY zzz_signals_w_engine_reverberation
    ADD CONSTRAINT zzz_signals_w_engine_reverberation_uid_fkey FOREIGN KEY (uid) REFERENCES zzz_uids (uid) ON DELETE CASCADE;

