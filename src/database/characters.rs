use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbCharacter {
    pub tag: String,
    pub name: String,
}

impl AsRef<DbCharacter> for DbCharacter {
    fn as_ref(&self) -> &DbCharacter {
        self
    }
}

pub async fn set_character(character: &DbCharacter, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters(tag, name)
        VALUES
            ($1, $2)
        ON CONFLICT
            (tag)
        DO UPDATE SET
            name = EXCLUDED.name
        ",
        character.tag,
        character.name,
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
