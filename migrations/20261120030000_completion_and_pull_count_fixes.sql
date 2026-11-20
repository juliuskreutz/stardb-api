-- Follow-ups to the running achievement totals and stored pull counts.

-- Qualifying players per game, kept by completion list writes so percentage reads no
-- longer count *_achievement_completion_totals on every request.
ALTER TABLE achievement_completion_state
    ADD COLUMN qualifying_players bigint NOT NULL DEFAULT 0 CHECK (qualifying_players >= 0);

UPDATE achievement_completion_state state
SET qualifying_players = CASE state.game
    WHEN 'hsr' THEN (SELECT COUNT(*) FROM achievement_completion_totals t WHERE t.completed >= state.minimum_completions)
    WHEN 'gi' THEN (SELECT COUNT(*) FROM gi_achievement_completion_totals t WHERE t.completed >= state.minimum_completions)
    WHEN 'zzz' THEN (SELECT COUNT(*) FROM zzz_achievement_completion_totals t WHERE t.completed >= state.minimum_completions)
END;

-- Deleting an account cascades away its completions without going through the list
-- writers; subtract a qualifying player's completions before that happens. Takes the
-- same shared lock as list writes so it cannot interleave with a recount, and updates
-- the state row before the counters, in the same order as list writes.
CREATE FUNCTION achievement_completions_forget_user() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    game_row record;
    minimum integer;
    qualifies boolean;
BEGIN
    FOR game_row IN SELECT * FROM (VALUES ('hsr', ''), ('gi', 'gi_'), ('zzz', 'zzz_')) AS g(game, prefix) LOOP
        PERFORM pg_advisory_xact_lock_shared(hashtext('achievement_completions'), hashtext(game_row.game));
        SELECT minimum_completions INTO minimum
        FROM achievement_completion_state WHERE game = game_row.game;
        CONTINUE WHEN NOT FOUND;
        EXECUTE format(
            'SELECT EXISTS (SELECT 1 FROM %I WHERE username = $1 AND completed >= $2)',
            game_row.prefix || 'achievement_completion_totals')
        INTO qualifies USING OLD.username, minimum;
        CONTINUE WHEN NOT qualifies;
        UPDATE achievement_completion_state
        SET qualifying_players = GREATEST(qualifying_players - 1, 0)
        WHERE game = game_row.game;
        -- Lock the counters in id order first, as list writes do, so the two cannot deadlock.
        EXECUTE format(
            'SELECT 1 FROM %I WHERE id IN (SELECT id FROM %I WHERE username = $1) ORDER BY id FOR UPDATE',
            game_row.prefix || 'achievements_percent',
            game_row.prefix || 'users_achievements_completed')
        USING OLD.username;
        EXECUTE format(
            'UPDATE %I AS counts SET completions = GREATEST(counts.completions - 1, 0) '
            'FROM %I AS c WHERE c.username = $1 AND c.id = counts.id',
            game_row.prefix || 'achievements_percent',
            game_row.prefix || 'users_achievements_completed')
        USING OLD.username;
    END LOOP;
    RETURN OLD;
END;
$$;

CREATE TRIGGER users_forget_achievement_completions
    BEFORE DELETE ON users
    FOR EACH ROW EXECUTE FUNCTION achievement_completions_forget_user();

-- Backfill and its hourly check find NULL pull counts through these partial indexes
-- instead of scanning each stats table; they are empty once every row is filled.
CREATE INDEX warps_stats_standard_pull_count_null_idx ON warps_stats_standard (uid) WHERE pull_count IS NULL;
CREATE INDEX warps_stats_special_pull_count_null_idx ON warps_stats_special (uid) WHERE pull_count IS NULL;
CREATE INDEX warps_stats_lc_pull_count_null_idx ON warps_stats_lc (uid) WHERE pull_count IS NULL;
CREATE INDEX warps_stats_collab_pull_count_null_idx ON warps_stats_collab (uid) WHERE pull_count IS NULL;
CREATE INDEX warps_stats_collab_lc_pull_count_null_idx ON warps_stats_collab_lc (uid) WHERE pull_count IS NULL;

CREATE INDEX gi_wishes_stats_standard_pull_count_null_idx ON gi_wishes_stats_standard (uid) WHERE pull_count IS NULL;
CREATE INDEX gi_wishes_stats_character_pull_count_null_idx ON gi_wishes_stats_character (uid) WHERE pull_count IS NULL;
CREATE INDEX gi_wishes_stats_weapon_pull_count_null_idx ON gi_wishes_stats_weapon (uid) WHERE pull_count IS NULL;
CREATE INDEX gi_wishes_stats_chronicled_pull_count_null_idx ON gi_wishes_stats_chronicled (uid) WHERE pull_count IS NULL;

CREATE INDEX zzz_signals_stats_standard_pull_count_null_idx ON zzz_signals_stats_standard (uid) WHERE pull_count IS NULL;
CREATE INDEX zzz_signals_stats_special_pull_count_null_idx ON zzz_signals_stats_special (uid) WHERE pull_count IS NULL;
CREATE INDEX zzz_signals_stats_w_engine_pull_count_null_idx ON zzz_signals_stats_w_engine (uid) WHERE pull_count IS NULL;
CREATE INDEX zzz_signals_stats_bangboo_pull_count_null_idx ON zzz_signals_stats_bangboo (uid) WHERE pull_count IS NULL;
CREATE INDEX zzz_signals_stats_exclusive_rescreening_pull_count_null_idx ON zzz_signals_stats_exclusive_rescreening (uid) WHERE pull_count IS NULL;
CREATE INDEX zzz_signals_stats_w_engine_reverberation_pull_count_null_idx ON zzz_signals_stats_w_engine_reverberation (uid) WHERE pull_count IS NULL;
