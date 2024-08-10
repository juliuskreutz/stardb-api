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

pub async fn set_all(id: &[i32], rarity: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            characters(id, rarity)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::integer[])
        ON CONFLICT
            (id)
        DO UPDATE SET
            rarity = EXCLUDED.rarity
        ",
        id,
        rarity,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(language: Language, pool: &PgPool) -> Result<Vec<DbCharacter>> {
    let language = language.to_string();

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
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbCharacter> {
    let language = language.to_string();

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
        language,
    )
    .fetch_one(pool)
    .await?)
}
