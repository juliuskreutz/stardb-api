use anyhow::Result;
use sqlx::PgPool;

pub async fn set_all(id: &[i32], rarity: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/characters/set_all.sql", id, rarity)
        .execute(pool)
        .await?;

    Ok(())
}
