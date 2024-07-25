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

    sqlx::query_file!("sql/gi/weapons_text/set_all.sql", id, language, name)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_id_by_name(name: &str, pool: &PgPool) -> Result<i32> {
    Ok(
        sqlx::query_file!("sql/gi/weapons_text/get_id_by_name.sql", name)
            .fetch_one(pool)
            .await?
            .id,
    )
}
