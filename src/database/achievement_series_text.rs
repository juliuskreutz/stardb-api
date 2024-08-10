use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub async fn set_all(
    id: &[i32],
    language: &[Language],
    name: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query!(
        "
        INSERT INTO
            achievement_series_text(id, language, name)
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
