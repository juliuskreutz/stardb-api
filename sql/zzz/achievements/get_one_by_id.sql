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
    zzz_achievements_text.name,
    zzz_achievements_text.description,
    COALESCE(percent, 0) percent,
    zzz_achievement_series_text.name series_name
FROM
    zzz_achievements
    LEFT JOIN zzz_achievements_percent ON zzz_achievements.id = zzz_achievements_percent.id
    JOIN zzz_achievements_text ON zzz_achievements.id = zzz_achievements_text.id
        AND zzz_achievements_text.language = $2
    JOIN zzz_achievement_series ON series = zzz_achievement_series.id
    JOIN zzz_achievement_series_text ON series = zzz_achievement_series_text.id
        AND zzz_achievement_series_text.language = $2
WHERE
    zzz_achievements.id = $1;

