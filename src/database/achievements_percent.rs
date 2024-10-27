use anyhow::Result;
use sqlx::PgPool;

const THRESHOLD: i64 = 600;

pub async fn update(pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/achievements_percent/update.sql", THRESHOLD)
        .execute(pool)
        .await?;

    Ok(())
}
