use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbBookSeriesWorld {
    pub id: i32,
    pub name: String,
}

pub async fn set_all_book_series_worlds(id: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            book_series_worlds(id)
        SELECT
            *
        FROM
            UNNEST($1::integer[])
        ON CONFLICT DO NOTHING
        ",
        id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_book_series_worlds(
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbBookSeriesWorld>> {
    let language = language.to_string();

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
    language: Language,
    pool: &PgPool,
) -> Result<DbBookSeriesWorld> {
    let language = language.to_string();

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
