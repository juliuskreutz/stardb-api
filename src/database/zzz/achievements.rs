use anyhow::Result;
use sqlx::PgPool;

pub async fn set_all(
    id: &[i32],
    series: &[i32],
    polychromes: &[i32],
    hidden: &[bool],
    priority: &[i32],
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/achievements/set_all.sql",
        id,
        series,
        polychromes,
        hidden,
        priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}
