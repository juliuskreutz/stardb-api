use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub async fn set_all(
    id: &[i32],
    language: &[Language],
    name: &[String],
    path: &[String],
    element: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query!(
        "
        INSERT INTO
            characters_text(id, language, name, path, element)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::text[], $3::text[], $4::text[], $5::text[])
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            path = EXCLUDED.path,
            element = EXCLUDED.element
        ",
        id,
        language,
        name,
        path,
        element,
    )
    .execute(pool)
    .await?;

    Ok(())
}
