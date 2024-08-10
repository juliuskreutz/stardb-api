use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbAchievementSeries {
    pub id: i32,
    pub name: String,
}

pub async fn set_all(id: &[i32], priority: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievement_series(id, priority)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::integer[])
        ON CONFLICT
            (id)
        DO UPDATE SET
            priority = EXCLUDED.priority
        ",
        id,
        priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(language: Language, pool: &PgPool) -> Result<Vec<DbAchievementSeries>> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbAchievementSeries,
        "
        SELECT
            achievement_series.id,
            achievement_series_text.name
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

pub async fn get_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbAchievementSeries> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbAchievementSeries,
        "
        SELECT
            achievement_series.id,
            achievement_series_text.name
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
