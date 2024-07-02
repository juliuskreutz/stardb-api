use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub async fn set_all_skill_texts(
    id: &[i32],
    language: &[Language],
    name: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query!(
        "
        INSERT INTO
            skills_text(id, language, name)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::text[], $3::text[])
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name
        ",
        id,
        language,
        name,
    )
    .execute(pool)
    .await?;

    Ok(())
}
