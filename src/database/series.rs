use anyhow::Result;
use sqlx::PgPool;

pub struct DbSeries {
    pub id: i32,
    pub name: String,
    pub priority: i32,
}

pub async fn set_series(series: &DbSeries, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            series(id, priority)
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

pub async fn get_series(language: &str, pool: &PgPool) -> Result<Vec<DbSeries>> {
    Ok(sqlx::query_as!(
        DbSeries,
        "
        SELECT
            series.id,
            series_text.name,
            series.priority
        FROM
            series
        INNER JOIN
            series_text
        ON
            series_text.id = series.id AND series_text.language = $1
        ORDER BY
            priority DESC, series.id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_series_by_id(id: i32, language: &str, pool: &PgPool) -> Result<DbSeries> {
    Ok(sqlx::query_as!(
        DbSeries,
        "
        SELECT
            series.id,
            series_text.name,
            series.priority
        FROM
            series
        INNER JOIN
            series_text
        ON
            series_text.id = series.id AND series_text.language = $2
        WHERE
            series.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
