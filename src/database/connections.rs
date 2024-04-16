use anyhow::Result;
use sqlx::PgPool;

pub struct DbConnection {
    pub uid: i32,
    pub username: String,
}

pub async fn set_connection(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO connections(uid, username) VALUES($1, $2)",
        connection.uid,
        connection.username,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_connection(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "DELETE FROM connections WHERE uid = $1 AND username = $2",
        connection.uid,
        connection.username,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_connections_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbConnection>> {
    Ok(sqlx::query_as!(
        DbConnection,
        "SELECT * FROM connections WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}
