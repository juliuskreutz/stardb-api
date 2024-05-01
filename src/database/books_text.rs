use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbBookText {
    pub id: i32,
    pub language: Language,
    pub name: String,
}

pub async fn set_book_text(book_text: &DbBookText, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            books_text(id, language, name)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name
        ",
        book_text.id,
        book_text.language as Language,
        book_text.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}
