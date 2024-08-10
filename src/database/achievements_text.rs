use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub async fn set_all(
    id: &[i32],
    language: &[Language],
    name: &[String],
    description: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query!(
        "
        INSERT INTO
            achievements_text(id, language, name, description)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::text[], $3::text[], $4::text[])
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description
        ",
        id,
        language,
        name,
        description,
    )
    .execute(pool)
    .await?;

    Ok(())
}
