use sqlx::PgPool;

use crate::Result;

pub struct DbCharacter {
    pub id: i32,
    pub tag: String,
    pub name: String,
    pub element: String,
    pub path: String,
}

pub async fn set_character(character: &DbCharacter, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters(id, tag, name, element, path)
        VALUES
            ($1, $2, $3, $4, $5)
        ON CONFLICT
            (id)
        DO UPDATE SET
            tag = EXCLUDED.tag,
            name = EXCLUDED.name,
            element = EXCLUDED.element,
            path = EXCLUDED.path
        ",
        character.id,
        character.tag,
        character.name,
        character.element,
        character.path,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_characters(pool: &PgPool) -> Result<Vec<DbCharacter>> {
    Ok(sqlx::query_as!(DbCharacter, "SELECT * FROM characters")
        .fetch_all(pool)
        .await?)
}
