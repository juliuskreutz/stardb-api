use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbCharacter {
    pub id: i32,
    pub rarity: i32,
    pub name: String,
    pub path: String,
    pub element: String,
    pub path_id: String,
    pub element_id: String,
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

pub async fn get_characters(language: Language, pool: &PgPool) -> Result<Vec<DbCharacter>> {
    Ok(sqlx::query_as!(
        DbCharacter,
        "
        SELECT
            characters.*,
            characters_text.name,
            characters_text.path,
            characters_text.element,
            characters_text_en.path path_id,
            characters_text_en.element element_id
        FROM
            characters
        INNER JOIN
            characters_text
        ON
            characters.id = characters_text.id AND characters_text.language = $1
        INNER JOIN
            characters_text AS characters_text_en
        ON
            characters.id = characters_text_en.id AND characters_text_en.language = 'en'
        ",
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_character_by_id(
    id: i32,
    language: Language,
    pool: &PgPool,
) -> Result<DbCharacter> {
    Ok(sqlx::query_as!(
        DbCharacter,
        "
        SELECT
            characters.*,
            characters_text.name,
            characters_text.path,
            characters_text.element,
            characters_text_en.path path_id,
            characters_text_en.element element_id
        FROM
            characters
        INNER JOIN
            characters_text
        ON
            characters.id = characters_text.id AND characters_text.language = $2
        INNER JOIN
            characters_text AS characters_text_en
        ON
            characters.id = characters_text_en.id AND characters_text_en.language = 'en'
        WHERE
            characters.id = $1
        ",
        id,
        language as Language,
    )
    .fetch_one(pool)
    .await?)
}
