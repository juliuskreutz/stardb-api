ALTER TABLE ntehelper_tracker_uid_claim
    DROP CONSTRAINT ntehelper_tracker_uid_claim_owner_user_id_fkey;

ALTER TABLE ntehelper_tracker_uid_claim
    ALTER COLUMN owner_user_id DROP NOT NULL;

ALTER TABLE ntehelper_tracker_uid_claim
    ADD CONSTRAINT ntehelper_tracker_uid_claim_owner_user_id_fkey
    FOREIGN KEY (owner_user_id) REFERENCES users (id) ON DELETE SET NULL;

DROP INDEX ntehelper_tracker_uid_claim_self_owner_idx;

CREATE INDEX ntehelper_tracker_uid_claim_self_owner_idx
    ON ntehelper_tracker_uid_claim (owner_user_id)
    WHERE claim_source = 'self';
