use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbBookSeries {
    pub id: i32,
    pub world: i32,
    pub bookshelf: bool,
    pub world_name: String,
    pub name: String,
}

pub async fn set_all_book_series(
    id: &[i32],
    world: &[i32],
    bookshelf: &[bool],
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            book_series(id, world, bookshelf)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::integer[], $3::boolean[])
        ON CONFLICT
            (id)
        DO UPDATE SET
            world = EXCLUDED.world,
            bookshelf = EXCLUDED.bookshelf
        ",
        id,
        world,
        bookshelf,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_book_series(language: Language, pool: &PgPool) -> Result<Vec<DbBookSeries>> {
    let language = language.to_string();

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
            book_series_worlds_text.id = book_series.world AND book_series_worlds_text.language = $1
        ORDER BY
            world, id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_book_series_by_id(
    id: i32,
    language: Language,
    pool: &PgPool,
) -> Result<DbBookSeries> {
    let language = language.to_string();

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
            book_series_worlds_text.id = book_series.world AND book_series_worlds_text.language = $2
        WHERE
            book_series.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
