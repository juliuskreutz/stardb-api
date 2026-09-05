use anyhow::Result;
use sqlx::PgPool;

use crate::Language;

/// Upsert catalog rows from aligned parallel slices; each index must describe the same record.
/// Database errors propagate to the catalog refresh caller.
pub async fn set_all(
    id: &[i32],
    language: &[Language],
    name: &[String],
    description: &[String],
    pool: &PgPool,
) -> Result<()> {
    let language = &language.iter().map(ToString::to_string).collect::<Vec<_>>();

    sqlx::query_file!(
        "sql/zzz/achievements_text/set_all.sql",
        id,
        language,
        name,
        description,
    )
    .execute(pool)
    .await?;

    Ok(())
}
