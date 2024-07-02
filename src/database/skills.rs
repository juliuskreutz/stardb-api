use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub struct DbSkill {
    pub id: i32,
    pub character: i32,
    pub name: String,
}

pub async fn set_all_skills(id: &[i32], character: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            skills(id, character)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::integer[])
        ON CONFLICT
            (id)
        DO UPDATE SET
            character = EXCLUDED.character
        ",
        id,
        character,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_skills(language: Language, pool: &PgPool) -> Result<Vec<DbSkill>> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbSkill,
        "
        SELECT
            skills.*,
            skills_text.name
        FROM
            skills
        INNER JOIN
            skills_text
        ON
            skills.id = skills_text.id AND skills_text.language = $1
        ORDER BY
            id
        ",
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_skill_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbSkill> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbSkill,
        "
        SELECT
            skills.*,
            skills_text.name
        FROM
            skills
        INNER JOIN
            skills_text
        ON
            skills.id = skills_text.id AND skills_text.language = $2
        WHERE
            skills.id = $1
        ",
        id,
        language,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_skills_by_character(
    character: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbSkill>> {
    let language = language.to_string();

    Ok(sqlx::query_as!(
        DbSkill,
        "
        SELECT
            skills.*,
            skills_text.name
        FROM
            skills
        INNER JOIN
            skills_text
        ON
            skills.id = skills_text.id AND skills_text.language = $2
        WHERE
            skills.character = $1
        ",
        character,
        language,
    )
    .fetch_all(pool)
    .await?)
}
