use anyhow::Result;
use sqlx::PgPool;

pub struct DbSeriesText {
    pub id: i32,
    pub language: String,
    pub name: String,
}

pub async fn set_series_text(series_text: &DbSeriesText, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            series_text(id, language, name)
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
