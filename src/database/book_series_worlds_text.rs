use anyhow::Result;
use sqlx::PgPool;

pub struct DbBookSeriesWorldText {
    pub id: i32,
    pub language: String,
    pub name: String,
}

pub async fn set_book_series_world_text(
    series_text: &DbBookSeriesWorldText,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            book_series_worlds_text(id, language, name)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name
        ",
        series_text.id,
        series_text.language,
        series_text.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}
