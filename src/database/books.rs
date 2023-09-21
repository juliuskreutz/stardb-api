use anyhow::Result;
use sqlx::PgPool;

#[derive(Clone)]
pub struct DbBook {
    pub id: i64,
    pub series: i32,
    pub series_name: String,
    pub series_world: i32,
    pub series_world_name: String,
    pub series_inside: i32,
    pub name: String,
    pub comment: Option<String>,
    pub image1: Option<String>,
    pub image2: Option<String>,
    pub percent: f64,
}

pub async fn set_book(book: &DbBook, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            books(id, series, series_inside)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id)
        DO UPDATE SET
            series = EXCLUDED.series,
            series_inside = EXCLUDED.series_inside
        ",
        book.id,
        book.series,
        book.series_inside
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_books(language: &str, pool: &PgPool) -> Result<Vec<DbBook>> {
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
        ORDER BY
            world, series, id
        ",
        language
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_book_by_id(id: i64, language: &str, pool: &PgPool) -> Result<DbBook> {
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
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_books_id(pool: &PgPool) -> Result<Vec<i64>> {
    Ok(sqlx::query!(
        "
        SELECT
            id
        FROM
            books
        "
    )
    .fetch_all(pool)
    .await?
    .iter()
    .map(|r| r.id)
    .collect())
}

pub async fn update_book_comment(id: i64, comment: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET comment = $2 WHERE id = $1", id, comment,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_book_image1(id: i64, image1: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image1 = $2 WHERE id = $1", id, image1,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_book_image2(id: i64, image2: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image2 = $2 WHERE id = $1", id, image2,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_book_comment(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET comment = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_book_image1(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image1 = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_book_image2(id: i64, pool: &PgPool) -> Result<()> {
    sqlx::query!("UPDATE books SET image2 = NULL WHERE id = $1", id)
        .execute(pool)
        .await?;

    Ok(())
}
