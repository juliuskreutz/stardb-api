use anyhow::Result;
use sqlx::PgPool;

pub struct DbCharacter {
    pub id: i32,
    pub rarity: i32,
    pub name: String,
    pub element: String,
    pub path: String,
}

pub async fn set_character(character: &DbCharacter, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters(id, rarity)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            rarity = EXCLUDED.rarity
        ",
        character.id,
        character.rarity,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_characters(language: &str, pool: &PgPool) -> Result<Vec<DbCharacter>> {
    Ok(sqlx::query_as!(
        DbCharacter,
        "
        SELECT
            characters.*,
            characters_text.name,
            characters_text.element,
            characters_text.path
        FROM
            characters
        INNER JOIN
            characters_text
        ON
            characters.id = characters_text.id AND characters_text.language = $1
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_character_by_id(id: i32, language: &str, pool: &PgPool) -> Result<DbCharacter> {
    Ok(sqlx::query_as!(
        DbCharacter,
        "
        SELECT
            characters.*,
            characters_text.name,
            characters_text.element,
            characters_text.path
        FROM
            characters
        INNER JOIN
            characters_text
        ON
            characters.id = characters_text.id AND characters_text.language = $2
        WHERE
            characters.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}
