CREATE TABLE ntehelper_tracker_uid_claim (
    uid bigint PRIMARY KEY,
    owner_user_id bigint NOT NULL,
    claim_source text NOT NULL,
    claimed_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ntehelper_tracker_uid_claim_owner_user_id_fkey FOREIGN KEY (owner_user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT ntehelper_tracker_uid_claim_uid_check CHECK (uid BETWEEN 100000 AND 999999999999),
    CONSTRAINT ntehelper_tracker_uid_claim_source_check CHECK (claim_source IN ('self', 'admin'))
);

CREATE UNIQUE INDEX ntehelper_tracker_uid_claim_self_owner_idx
    ON ntehelper_tracker_uid_claim (owner_user_id)
    WHERE claim_source = 'self';

CREATE INDEX ntehelper_tracker_uid_claim_owner_idx
    ON ntehelper_tracker_uid_claim (owner_user_id, claimed_at DESC);

CREATE TABLE ntehelper_tracker_pull (
    uid bigint NOT NULL,
    record_uid text NOT NULL,
    pool_group_id text NOT NULL,
    banner_type text NOT NULL,
    timestamp_raw text NOT NULL,
    timestamp_group_ordinal integer,
    roll_result integer,
    result_type text,
    reward_type text NOT NULL,
    reward_id text NOT NULL,
    reward_name text NOT NULL,
    reward_rank text,
    star_rank integer,
    quantity integer,
    imported_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (uid, record_uid),
    CONSTRAINT ntehelper_tracker_pull_uid_fkey FOREIGN KEY (uid) REFERENCES ntehelper_tracker_uid_claim (uid) ON DELETE CASCADE,
    CONSTRAINT ntehelper_tracker_pull_record_uid_check CHECK (
        char_length(btrim(record_uid)) BETWEEN 1 AND 128
        AND btrim(record_uid) !~ '[^A-Za-z0-9_.:-]'
    ),
    CONSTRAINT ntehelper_tracker_pull_pool_group_check CHECK (pool_group_id IN ('Lottery_LimitedCharacter', 'Lottery_Permanent', 'Arc_MiracleBox')),
    CONSTRAINT ntehelper_tracker_pull_banner_type_check CHECK (banner_type IN ('limited-character', 'permanent-character', 'arc')),
    CONSTRAINT ntehelper_tracker_pull_timestamp_raw_check CHECK (timestamp_raw ~ '^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$'),
    CONSTRAINT ntehelper_tracker_pull_timestamp_group_ordinal_check CHECK (timestamp_group_ordinal IS NULL OR timestamp_group_ordinal >= 0),
    CONSTRAINT ntehelper_tracker_pull_roll_result_check CHECK (roll_result IS NULL OR roll_result >= 0),
    CONSTRAINT ntehelper_tracker_pull_result_type_check CHECK (result_type IS NULL OR char_length(btrim(result_type)) BETWEEN 1 AND 64),
    CONSTRAINT ntehelper_tracker_pull_reward_type_check CHECK (char_length(btrim(reward_type)) BETWEEN 1 AND 64),
    CONSTRAINT ntehelper_tracker_pull_reward_id_check CHECK (char_length(btrim(reward_id)) BETWEEN 1 AND 128),
    CONSTRAINT ntehelper_tracker_pull_reward_name_check CHECK (char_length(btrim(reward_name)) BETWEEN 1 AND 128),
    CONSTRAINT ntehelper_tracker_pull_reward_rank_check CHECK (reward_rank IS NULL OR reward_rank IN ('S', 'A', 'B')),
    CONSTRAINT ntehelper_tracker_pull_star_rank_check CHECK (star_rank IS NULL OR star_rank IN (3, 4, 5)),
    CONSTRAINT ntehelper_tracker_pull_quantity_check CHECK (quantity IS NULL OR quantity >= 0)
);

CREATE INDEX ntehelper_tracker_pull_uid_timestamp_idx
    ON ntehelper_tracker_pull (uid, timestamp_raw DESC);

CREATE INDEX ntehelper_tracker_pull_uid_banner_timestamp_idx
    ON ntehelper_tracker_pull (uid, banner_type, timestamp_raw DESC);


