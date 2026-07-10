DROP INDEX IF EXISTS ntehelper_tracker_pull_uid_banner_timestamp_idx;

ALTER TABLE ntehelper_tracker_pull
    DROP CONSTRAINT IF EXISTS ntehelper_tracker_pull_reward_type_check,
    DROP CONSTRAINT IF EXISTS ntehelper_tracker_pull_reward_name_check,
    DROP CONSTRAINT IF EXISTS ntehelper_tracker_pull_star_rank_check;

ALTER TABLE ntehelper_tracker_pull
    DROP COLUMN IF EXISTS banner_type,
    DROP COLUMN IF EXISTS reward_type,
    DROP COLUMN IF EXISTS reward_name,
    DROP COLUMN IF EXISTS reward_rank,
    DROP COLUMN IF EXISTS star_rank;
