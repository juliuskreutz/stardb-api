use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbCharacterText {
    pub id: i32,
    pub language: Language,
    pub name: String,
    pub path: String,
    pub element: String,
}

pub async fn set_character_text(character_text: &DbCharacterText, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters_text(id, language, name, path, element)
        VALUES
            ($1, $2, $3, $4, $5)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            path = EXCLUDED.path,
            element = EXCLUDED.element
        ",
        character_text.id,
        character_text.language as Language,
        character_text.name,
        character_text.path,
        character_text.element,
    )
    .execute(pool)
    .await?;

    Ok(())
}
