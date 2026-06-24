ALTER TABLE ntehelper_tracker_pull
    DROP CONSTRAINT IF EXISTS ntehelper_tracker_pull_pool_group_check,
    DROP CONSTRAINT IF EXISTS ntehelper_tracker_pull_banner_type_check,
    DROP CONSTRAINT IF EXISTS ntehelper_tracker_pull_reward_rank_check;
