-- Precomputed achievement leaderboard ranks. Ranking every player with two window
-- functions on each page view cost ~1s on production; a background job now refreshes
-- this table every few minutes and pages read it through the indexes below.
-- Ranks match the previous query: achievement count descending, then the earliest
-- score timestamp, with RANK() semantics for exact ties.

CREATE TABLE achievement_ranks (
    uid integer PRIMARY KEY REFERENCES mihomo (uid) ON DELETE CASCADE,
    region text NOT NULL,
    global_rank bigint NOT NULL,
    regional_rank bigint NOT NULL
);

CREATE INDEX achievement_ranks_global_idx ON achievement_ranks (global_rank, uid);
CREATE INDEX achievement_ranks_region_idx ON achievement_ranks (region, global_rank, uid);

INSERT INTO achievement_ranks (uid, region, global_rank, regional_rank)
SELECT
    uid,
    region,
    RANK() OVER (ORDER BY achievement_count DESC, timestamp),
    RANK() OVER (PARTITION BY region ORDER BY achievement_count DESC, timestamp)
FROM
    scores_achievement
    NATURAL JOIN mihomo;
