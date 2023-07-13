use sqlx::PgPool;

use crate::Result;

pub struct DbComplete {
    pub username: String,
    pub id: i64,
}

pub async fn add_complete(complete: &DbComplete, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO completed(username, id) VALUES($1, $2)",
        complete.username,
        complete.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_complete(complete: &DbComplete, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "DELETE FROM completed WHERE username = $1 AND id = $2",
        complete.username,
        complete.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_completed_by_username(username: &str, pool: &PgPool) -> Result<Vec<DbComplete>> {
    Ok(sqlx::query_as!(
        DbComplete,
        "SELECT * FROM completed WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}
