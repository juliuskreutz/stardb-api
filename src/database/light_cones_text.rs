use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

pub async fn set_all(
    id: &[i32],
    language: &[Language],
    name: &[String],
    path: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query_file!(
        "sql/light_cones_texts/set_all.sql",
        id,
        language,
        name,
        path,
    )
    .execute(pool)
    .await?;

    Ok(())
}
