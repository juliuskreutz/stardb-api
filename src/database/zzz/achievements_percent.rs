use anyhow::Result;
use sqlx::PgPool;

const THRESHOLD: i64 = 0;

pub async fn update(pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/zzz/achievements_percent/update.sql", THRESHOLD)
        .execute(pool)
        .await?;

    Ok(())
}
