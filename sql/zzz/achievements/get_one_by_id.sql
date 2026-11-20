SELECT
    zzz_achievements.id,
    zzz_achievements.series,
    zzz_achievements.polychromes,
    zzz_achievements.hidden,
    zzz_achievements.version,
    zzz_achievements.comment,
    zzz_achievements.reference,
    zzz_achievements.difficulty,
    zzz_achievements.video,
    zzz_achievements.gacha,
    zzz_achievements.timegated,
    zzz_achievements.missable,
    zzz_achievements.impossible,
    zzz_achievements.set,
    zzz_achievements.arcade,
    zzz_achievements_text.name,
    zzz_achievements_text.description,
    -- Live share of qualifying players; the stored percent until the first recount.
    COALESCE(zzz_achievements_percent.completions::float8 / NULLIF(qualifying.players, 0), zzz_achievements_percent.percent, 0) percent,
    zzz_achievement_series_text.name series_name
FROM
    zzz_achievements
    LEFT JOIN zzz_achievements_percent ON zzz_achievements.id = zzz_achievements_percent.id
    CROSS JOIN (
        SELECT COUNT(*) AS players
        FROM achievement_completion_state state
        JOIN zzz_achievement_completion_totals totals ON totals.completed >= state.minimum_completions
        WHERE state.game = 'zzz'
    ) qualifying
    JOIN zzz_achievements_text ON zzz_achievements.id = zzz_achievements_text.id
        AND zzz_achievements_text.language = $2
    JOIN zzz_achievement_series ON series = zzz_achievement_series.id
    JOIN zzz_achievement_series_text ON series = zzz_achievement_series_text.id
        AND zzz_achievement_series_text.language = $2
WHERE
    zzz_achievements.id = $1;

