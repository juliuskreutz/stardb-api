use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

#[derive(Clone)]
pub struct DbBook {
    pub id: i32,
    pub series: i32,
    pub series_name: String,
    pub series_world: i32,
    pub series_world_name: String,
    pub series_inside: i32,
    pub icon: Option<i32>,
    pub name: String,
    pub comment: Option<String>,
    pub image1: Option<String>,
    pub image2: Option<String>,
    pub percent: f64,
}

pub async fn set_all_books(
    id: &[i32],
    series: &[i32],
    series_inside: &[i32],
    icon: &[i32],
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            books(id, series, series_inside, icon)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::integer[], $3::integer[], $4::integer[])
        ON CONFLICT
            (id)
        DO UPDATE SET
            series = EXCLUDED.series,
            series_inside = EXCLUDED.series_inside,
            icon = EXCLUDED.icon
        ",
        id,
        series,
        series_inside,
        icon,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_books(language: Language, pool: &PgPool) -> Result<Vec<DbBook>> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbBook,
        "
        SELECT
            books.*,
            books_text.name,
            percent,
            book_series.world series_world,
            book_series_text.name series_name,
            book_series_worlds_text.name series_world_name
        FROM
            books
        NATURAL INNER JOIN
            books_percent
        INNER JOIN
            books_text
        ON
            books.id = books_text.id AND books_text.language = $1
        INNER JOIN
            book_series
        ON
            series = book_series.id
        INNER JOIN
            book_series_text
        ON
            series = book_series_text.id AND book_series_text.language = $1
        INNER JOIN
            book_series_worlds_text
        ON
            book_series.world = book_series_worlds_text.id AND book_series_worlds_text.language = $1
        WHERE
            icon IS NOT NULL
        ORDER BY
            world, series, series_inside, id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_book_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbBook> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbBook,
        "
        SELECT
            books.*,
            books_text.name,
            percent,
            book_series.world series_world,
            book_series_text.name series_name,
            book_series_worlds_text.name series_world_name
        FROM
            books
        NATURAL INNER JOIN
            books_percent
        INNER JOIN
            books_text
        ON
            books.id = books_text.id AND books_text.language = $2
        INNER JOIN
            book_series
        ON
            series = book_series.id
        INNER JOIN
            book_series_text
        ON
            series = book_series_text.id AND book_series_text.language = $2
        INNER JOIN
            book_series_worlds_text
        ON
            book_series.world = book_series_worlds_text.id AND book_series_worlds_text.language = $2
        WHERE
            books.id = $1
        AND
            icon IS NOT NULL
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update_book_comment(id: i32, comment: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET comment = $2 WHERE id = $1", id, comment,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_book_image1(id: i32, image1: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image1 = $2 WHERE id = $1", id, image1,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_book_image2(id: i32, image2: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image2 = $2 WHERE id = $1", id, image2,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_book_comment(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET comment = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_book_image1(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image1 = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_book_image2(id: i32, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image2 = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}
