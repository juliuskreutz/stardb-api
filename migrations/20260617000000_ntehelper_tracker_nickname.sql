ALTER TABLE ntehelper_tracker_uid_claim
    ADD COLUMN nickname text NOT NULL DEFAULT '';

ALTER TABLE ntehelper_tracker_uid_claim
    ADD CONSTRAINT ntehelper_tracker_uid_claim_nickname_check CHECK (
        char_length(btrim(nickname)) <= 24
        AND btrim(nickname) !~ '[[:cntrl:]]'
    );
