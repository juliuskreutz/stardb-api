use sqlx::PgPool;

use anyhow::Result;

pub struct DbAdmin {
    pub username: String,
}

pub async fn get_admin_by_username(username: &str, pool: &PgPool) -> Result<DbAdmin> {
    Ok(sqlx::query_as!(
        DbAdmin,
        "SELECT * FROM admins WHERE username = $1",
        username
    )
    .fetch_one(pool)
    .await?)
}
