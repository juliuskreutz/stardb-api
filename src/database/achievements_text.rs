use anyhow::Result;
use sqlx::PgPool;

pub struct DbAchievementText {
    pub id: i32,
    pub language: String,
    pub name: String,
    pub description: String,
}

pub async fn set_achievement_text(
    achievement_text: &DbAchievementText,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            achievements_text(id, language, name, description)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description
        ",
        achievement_text.id,
        achievement_text.language,
        achievement_text.name,
        achievement_text.description,
    )
    .execute(pool)
    .await?;

    Ok(())
}
