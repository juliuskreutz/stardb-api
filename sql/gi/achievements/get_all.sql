SELECT
    gi_achievements.id,
    gi_achievements.series,
    gi_achievements.primogems,
    gi_achievements.hidden,
    gi_achievements.version,
    gi_achievements.comment,
    gi_achievements.reference,
    gi_achievements.difficulty,
    gi_achievements.video,
    gi_achievements.gacha,
    gi_achievements.timegated,
    gi_achievements.missable,
    gi_achievements.impossible,
    gi_achievements.set,
    gi_achievements_text.name,
    gi_achievements_text.description,
    percent,
    gi_achievement_series_text.name series_name
FROM
    gi_achievements
    JOIN gi_achievements_percent ON gi_achievements.id = gi_achievements_percent.id
    JOIN gi_achievements_text ON gi_achievements.id = gi_achievements_text.id
        AND gi_achievements_text.language = $1
    JOIN gi_achievement_series ON series = gi_achievement_series.id
    JOIN gi_achievement_series_text ON series = gi_achievement_series_text.id
        AND gi_achievement_series_text.language = $1
    ORDER BY
        gi_achievement_series.priority,
        gi_achievements.priority,
        id;

