-- Running totals for achievement completion percentages. The hourly percent jobs
-- re-aggregated every completion (~77M rows, ~5 minutes per hour on production).
-- Completion list writes now keep these totals current; a full recount runs weekly
-- (and once after deploy) to correct any drift.
--
-- completions: completions of the achievement by qualifying players. NULL until the
--   game's first full recount, so readers keep serving the stored percent until then.
-- *_achievement_completion_totals: completed achievements per player, for the
--   qualifying-player threshold (HSR counts players with 600+ completions).
-- achievement_completion_state: one row per game once it has been recounted, with
--   the threshold the totals were built for.

ALTER TABLE achievements_percent ADD COLUMN completions bigint CHECK (completions >= 0);
ALTER TABLE gi_achievements_percent ADD COLUMN completions bigint CHECK (completions >= 0);
ALTER TABLE zzz_achievements_percent ADD COLUMN completions bigint CHECK (completions >= 0);

CREATE TABLE achievement_completion_totals (
    username text PRIMARY KEY REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE,
    completed integer NOT NULL CHECK (completed > 0)
);
CREATE INDEX achievement_completion_totals_completed_idx ON achievement_completion_totals (completed);

CREATE TABLE gi_achievement_completion_totals (
    username text PRIMARY KEY REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE,
    completed integer NOT NULL CHECK (completed > 0)
);
CREATE INDEX gi_achievement_completion_totals_completed_idx ON gi_achievement_completion_totals (completed);

CREATE TABLE zzz_achievement_completion_totals (
    username text PRIMARY KEY REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE,
    completed integer NOT NULL CHECK (completed > 0)
);
CREATE INDEX zzz_achievement_completion_totals_completed_idx ON zzz_achievement_completion_totals (completed);

CREATE TABLE achievement_completion_state (
    game text PRIMARY KEY CHECK (game IN ('hsr', 'gi', 'zzz')),
    minimum_completions integer NOT NULL,
    recomputed_at timestamp with time zone NOT NULL
);
