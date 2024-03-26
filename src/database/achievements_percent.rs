use anyhow::Result;
use sqlx::PgPool;

const THRESHOLD: i64 = 300;

pub async fn update_achievements_percent(pool: &PgPool) -> Result<()> {
    let mut transaction = pool.begin().await?;

    sqlx::query!("TRUNCATE achievements_percent")
        .execute(&mut *transaction)
        .await?;

    sqlx::query!(
        "
        WITH threshholded_users_achievements AS (
            SELECT
                users_achievements.username,
                id
            FROM
                users_achievements
            JOIN
                (SELECT username FROM users_achievements GROUP BY username HAVING count(*) >= $1) threshholded_users
            ON
                users_achievements.username = threshholded_users.username
        ), achievements_percent AS (
            SELECT
                id,
                COUNT(*)::float / (
                    SELECT COUNT(*) FROM users WHERE EXISTS (
                        SELECT * FROM threshholded_users_achievements WHERE users.username = threshholded_users_achievements.username
                    )
                ) percent
            FROM
                threshholded_users_achievements
            GROUP BY
                id
        )
        INSERT INTO
            achievements_percent(id, percent)
        SELECT
            achievements.id,
            COALESCE(percent, 0)
        FROM 
            achievements
        LEFT JOIN
            achievements_percent
        ON
            achievements.id = achievements_percent.id
        ",
        THRESHOLD,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
