use anyhow::Result;
use sqlx::PgPool;

/// Upsert catalog rows from aligned parallel slices; each index must describe the same record.
/// Database errors propagate to the catalog refresh caller.
pub async fn set_all(id: &[i32], priority: &[i32], pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/gi/achievement_series/set_all.sql", id, priority,)
        .execute(pool)
        .await?;

    Ok(())
}
