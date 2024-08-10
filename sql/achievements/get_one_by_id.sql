SELECT
    achievements.id,
    achievements.series,
    achievements.jades,
    achievements.hidden,
    achievements.version,
    achievements.comment,
    achievements.reference,
    achievements.difficulty,
    achievements.video,
    achievements.gacha,
    achievements.timegated,
    achievements.missable,
    achievements.impossible,
    achievements.set,
    achievements_text.name,
    achievements_text.description,
    percent,
    achievement_series_text.name series_name
FROM
    achievements
    JOIN achievements_percent ON achievements.id = achievements_percent.id
    JOIN achievements_text ON achievements.id = achievements_text.id
        AND achievements_text.language = $2
    JOIN achievement_series ON series = achievement_series.id
    JOIN achievement_series_text ON series = achievement_series_text.id
        AND achievement_series_text.language = $2
WHERE
    achievements.id = $1;

