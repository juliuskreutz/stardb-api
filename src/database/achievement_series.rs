use anyhow::Result;
use sqlx::PgPool;

pub struct DbAchievementSeries {
    pub id: i32,
    pub name: String,
    pub priority: i32,
}

pub async fn set_series(series: &DbAchievementSeries, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievement_series(id, priority)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            priority = EXCLUDED.priority
        ",
        series.id,
        series.priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_series(language: &str, pool: &PgPool) -> Result<Vec<DbAchievementSeries>> {
    Ok(sqlx::query_as!(
        DbAchievementSeries,
        "
        SELECT
            achievement_series.id,
            achievement_series_text.name,
            achievement_series.priority
        FROM
            achievement_series
        INNER JOIN
            achievement_series_text
        ON
            achievement_series_text.id = achievement_series.id AND achievement_series_text.language = $1
        ORDER BY
            priority DESC, achievement_series.id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_series_by_id(
    id: i32,
    language: &str,
    pool: &PgPool,
) -> Result<DbAchievementSeries> {
    Ok(sqlx::query_as!(
        DbAchievementSeries,
        "
        SELECT
            achievement_series.id,
            achievement_series_text.name,
            achievement_series.priority
        FROM
            achievement_series
        INNER JOIN
            achievement_series_text
        ON
            achievement_series_text.id = achievement_series.id AND achievement_series_text.language = $2
        WHERE
            achievement_series.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
