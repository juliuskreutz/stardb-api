use anyhow::Result;
use sqlx::PgPool;

pub struct DbAchievementPercent {
    pub id: i64,
    pub percent: f64,
}

pub async fn set_achievement_percent(
    achievement_percent: &DbAchievementPercent,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievements_percent(id, percent)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            percent = EXCLUDED.percent
        ",
        achievement_percent.id,
        achievement_percent.percent,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// pub async fn update_achievements_percent(pool: &PgPool) -> Result<()> {
//     sqlx::query!(
//         "
//         INSERT INTO
//             achievements_percent(id, percent)
//         SELECT
//             *
//         FROM
//             (SELECT
//                 id,
//                 (COUNT(completed.id)::float) / (SELECT COUNT(DISTINCT username) FROM completed) percent
//             FROM
//                 achievements
//             NATURAL LEFT JOIN
//                 completed
//             GROUP BY
//                 id
//             ) percents
//         ON CONFLICT
//             (id)
//         DO UPDATE SET
//             percent = EXCLUDED.percent
//         "
//     )
//     .execute(pool)
//     .await?;

//     Ok(())
// }
