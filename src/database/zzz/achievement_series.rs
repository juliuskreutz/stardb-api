use anyhow::Result;
use sqlx::PgPool;

pub async fn set_all(id: &[i32], priority: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/zzz/achievement_series/set_all.sql", id, priority)
        .execute(pool)
        .await?;

    Ok(())
}
