use sqlx::PgPool;

use anyhow::Result;

pub async fn update_achievements_percent(pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievements_percent(id, percent)
        SELECT
            *
        FROM
            (SELECT
                id,
                (COUNT(*)::float) / (SELECT COUNT(DISTINCT username) from completed) percent
            FROM
                completed
            GROUP BY
                id
            ) percents
        ON CONFLICT
            (id)
        DO UPDATE SET
            percent = EXCLUDED.percent
        "
    )
    .execute(pool)
    .await?;

    Ok(())
}
