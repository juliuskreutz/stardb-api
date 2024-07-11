SELECT
    zzz_achievements.*,
    zzz_achievements_text.name,
    zzz_achievements_text.description,
    percent,
    zzz_achievement_series_text.name series_name
FROM
    zzz_achievements
    JOIN zzz_achievements_percent ON zzz_achievements.id = zzz_achievements_percent.id
    JOIN zzz_achievements_text ON zzz_achievements.id = zzz_achievements_text.id
        AND zzz_achievements_text.language = $1
    JOIN zzz_achievement_series ON series = zzz_achievement_series.id
    JOIN zzz_achievement_series_text ON series = zzz_achievement_series_text.id
        AND zzz_achievement_series_text.language = $1
    ORDER BY
        series,
        priority DESC,
        id;

