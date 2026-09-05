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

    sqlx::query_file!("sql/gi/characters_text/set_all.sql", id, language, name)
        .execute(pool)
        .await?;

    Ok(())
}

/// Snapshot localized names once for an import job.
pub async fn get_name_ids(pool: &PgPool) -> Result<std::collections::HashMap<String, i32>> {
    Ok(sqlx::query_file!("sql/gi/characters_text/get_name_ids.sql")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| (row.name, row.id))
        .collect())
}
