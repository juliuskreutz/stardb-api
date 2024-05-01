use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbAchievementSeriesText {
    pub id: i32,
    pub language: Language,
    pub name: String,
}

pub async fn set_achievement_series_text(
    series_text: &DbAchievementSeriesText,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievement_series_text(id, language, name)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name
        ",
        series_text.id,
        series_text.language as Language,
        series_text.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}
