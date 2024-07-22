use anyhow::Result;
use sqlx::PgPool;

pub struct DbAdmin {
    pub username: String,
}

pub async fn get_one_by_username(username: &str, pool: &PgPool) -> Result<DbAdmin> {
    Ok(
        sqlx::query_file_as!(DbAdmin, "sql/admins/get_one_by_username.sql", username)
            .fetch_one(pool)
            .await?,
    )
}
