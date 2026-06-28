ALTER TABLE ntehelper_tracker_uid_claim
ADD COLUMN region text NOT NULL DEFAULT 'europe';

ALTER TABLE ntehelper_tracker_uid_claim
ADD CONSTRAINT ntehelper_tracker_uid_claim_region_check
CHECK (region IN ('asia', 'europe', 'america', 'china'));
