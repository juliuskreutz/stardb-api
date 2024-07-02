use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub async fn set_all_light_cone_texts(
    id: &[i32],
    language: &[Language],
    name: &[String],
    path: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query!(
        "
        INSERT INTO
            light_cones_text(id, language, name, path)
        SELECT
            *
        FROM
            UNNEST($1::integer[], $2::text[], $3::text[], $4::text[])
        ON CONFLICT
            (id, language)
        DO UPDATE SET
            name = EXCLUDED.name,
            path = EXCLUDED.path
        ",
        id,
        language,
        name,
        path,
    )
    .execute(pool)
    .await?;

    Ok(())
}
