use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbSeries {
    pub id: i32,
    pub name: String,
    pub priority: i32,
}

pub async fn set_series(series: &DbSeries, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            series(id, name, priority)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id)
        DO UPDATE SET
            name = EXCLUDED.name,
            priority = EXCLUDED.priority
        ",
        series.id,
        series.name,
        series.priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_series(pool: &PgPool) -> Result<Vec<DbSeries>> {
    Ok(sqlx::query_as!(
        DbSeries,
        "
        SELECT
            *
        FROM
            series
        ORDER BY
            priority DESC
        "
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_series_by_id(id: i32, pool: &PgPool) -> Result<DbSeries> {
    Ok(sqlx::query_as!(
        DbSeries,
        "
        SELECT
            *
        FROM
            series
        WHERE
            id = $1
        ",
        id,
    )
    .fetch_one(pool)
    .await?)
}
