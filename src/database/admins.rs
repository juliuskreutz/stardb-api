use anyhow::Result;
use sqlx::PgPool;

pub async fn exists(username: &str, pool: &PgPool) -> Result<bool> {
    Ok(sqlx::query_file!("sql/admins/exists.sql", username)
        .fetch_optional(pool)
        .await?
        .is_some())
}
