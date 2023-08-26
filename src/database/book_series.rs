use anyhow::Result;
use sqlx::PgPool;

pub struct DbBookSeries {
    pub id: i32,
    pub world: i32,
    pub world_name: String,
    pub name: String,
}

pub async fn set_book_series(series: &DbBookSeries, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            book_series(id, world)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            world = EXCLUDED.world
        ",
        series.id,
        series.world,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_book_series(language: &str, pool: &PgPool) -> Result<Vec<DbBookSeries>> {
    Ok(sqlx::query_as!(
        DbBookSeries,
        "
        SELECT
            book_series.*,
            book_series_text.name,
            book_series_worlds_text.name world_name
        FROM
            book_series
        INNER JOIN
            book_series_text
        ON
            book_series_text.id = book_series.id AND book_series_text.language = $1
        INNER JOIN
            book_series_worlds_text
        ON
            book_series_worlds_text.id = book_series.id AND book_series_worlds_text.language = $1
        ORDER BY
            id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_book_series_by_id(id: i32, language: &str, pool: &PgPool) -> Result<DbBookSeries> {
    Ok(sqlx::query_as!(
        DbBookSeries,
        "
        SELECT
            book_series.*,
            book_series_text.name,
            book_series_worlds_text.name world_name
        FROM
            book_series
        INNER JOIN
            book_series_text
        ON
            book_series_text.id = book_series.id AND book_series_text.language = $2
        INNER JOIN
            book_series_worlds_text
        ON
            book_series_worlds_text.id = book_series.id AND book_series_worlds_text.language = $2
        WHERE
            book_series.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
