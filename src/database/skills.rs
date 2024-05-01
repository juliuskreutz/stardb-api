use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

#[derive(Clone)]
pub struct DbSkill {
    pub id: i32,
    pub character: i32,
    pub name: String,
}

pub async fn set_skill(skill: &DbSkill, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            skills(id, character)
        VALUES
            ($1, $2)
        ON CONFLICT
            (id)
        DO UPDATE SET
            character = $2
        ",
        skill.id,
        skill.character,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_skills(language: Language, pool: &PgPool) -> Result<Vec<DbSkill>> {
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
        language as Language
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_skill_by_id(id: i32, language: Language, pool: &PgPool) -> Result<DbSkill> {
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
        language as Language,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_skills_by_character(
    character: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbSkill>> {
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
        language as Language,
    )
    .fetch_all(pool)
    .await?)
}
