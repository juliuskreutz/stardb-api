use anyhow::Result;
use sqlx::PgPool;

const THRESHOLD: i64 = 300;

pub async fn update(pool: &PgPool) -> Result<()> {
    let mut transaction = pool.begin().await?;

    sqlx::query_file!("sql/achievements_percent/truncate.sql")
        .execute(&mut *transaction)
        .await?;

    sqlx::query_file!("sql/achievements_percent/update.sql", THRESHOLD)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}
