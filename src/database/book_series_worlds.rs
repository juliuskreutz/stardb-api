use anyhow::Result;
use sqlx::PgPool;

pub struct DbBookSeriesWorld {
    pub id: i32,
    pub name: String,
}

pub async fn set_book_series_world(series: &DbBookSeriesWorld, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            book_series_worlds(id)
        VALUES
            ($1)
        ON CONFLICT DO NOTHING
        ",
        series.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_book_series_worlds(
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbBookSeriesWorld>> {
    Ok(sqlx::query_as!(
        DbBookSeriesWorld,
        "
        SELECT
            book_series_worlds.id,
            book_series_worlds_text.name
        FROM
            book_series_worlds
        INNER JOIN
            book_series_worlds_text
        ON
            book_series_worlds_text.id = book_series_worlds.id AND book_series_worlds_text.language = $1
        ORDER BY
            id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_book_series_world_by_id(
    id: i32,
    language: &str,
    pool: &PgPool,
) -> Result<DbBookSeriesWorld> {
    Ok(sqlx::query_as!(
        DbBookSeriesWorld,
        "
        SELECT
            book_series_worlds.id,
            book_series_worlds_text.name
        FROM
            book_series_worlds
        INNER JOIN
            book_series_worlds_text
        ON
            book_series_worlds_text.id = book_series_worlds.id AND book_series_worlds_text.language = $2
        WHERE
            book_series_worlds.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
