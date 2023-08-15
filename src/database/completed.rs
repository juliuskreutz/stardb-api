use sqlx::PgPool;

use anyhow::Result;

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

    if let Some(set) = sqlx::query!("SELECT set FROM achievements WHERE id = $1", complete.id,)
        .fetch_one(pool)
        .await?
        .set
    {
        for related in super::get_related(complete.id, set, pool).await? {
            sqlx::query!(
                "DELETE FROM completed WHERE username = $1 AND id = $2",
                complete.username,
                related,
            )
            .execute(pool)
            .await?;
        }
    }

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

pub async fn get_distinct_username_count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query!("SELECT COUNT(DISTINCT username) FROM completed")
            .fetch_one(pool)
            .await?
            .count
            .unwrap_or_default(),
    )
}
