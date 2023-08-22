use anyhow::Result;
use sqlx::PgPool;

pub struct DbCharacterText {
    pub id: i32,
    pub language: String,
    pub name: String,
    pub element: String,
    pub path: String,
}

pub async fn set_character_text(character_text: &DbCharacterText, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters_text(id, language, name, element, path)
        VALUES
            ($1, $2, $3, $4, $5)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            element = EXCLUDED.element,
            path = EXCLUDED.path
        ",
        character_text.id,
        character_text.language,
        character_text.name,
        character_text.element,
        character_text.path,
    )
    .execute(pool)
    .await?;

    Ok(())
}
