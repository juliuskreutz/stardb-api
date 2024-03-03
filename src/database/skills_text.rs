use anyhow::Result;
use sqlx::PgPool;

pub struct DbSkillText {
    pub id: i32,
    pub language: String,
    pub name: String,
}

pub async fn set_skill_text(skill_text: &DbSkillText, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            skills_text(id, language, name)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name
        ",
        skill_text.id,
        skill_text.language,
        skill_text.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}
