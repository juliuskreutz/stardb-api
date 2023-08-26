use anyhow::Result;
use sqlx::PgPool;

pub struct DbBookPercent {
    pub id: i64,
    pub percent: f64,
}

pub async fn set_book_percent(book_percent: &DbBookPercent, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            books_percent(id, percent)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            percent = EXCLUDED.percent
        ",
        book_percent.id,
        book_percent.percent,
    )
    .execute(pool)
    .await?;

    Ok(())
}
