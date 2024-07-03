use anyhow::Result;
use sqlx::PgPool;

pub struct DbConnection {
    pub uid: i32,
    pub username: String,
    pub verified: bool,
    pub private: bool,
}

pub async fn set_connection(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO connections
            (uid, username, verified, private) 
        VALUES
            ($1, $2, $3, $4) 
        ON CONFLICT
            (uid, username) 
        DO UPDATE SET 
            verified = EXCLUDED.verified
        ",
        connection.uid,
        connection.username,
        connection.verified,
        false,
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

pub async fn get_connections_by_uid(uid: i32, pool: &PgPool) -> Result<Vec<DbConnection>> {
    Ok(sqlx::query_as!(
        DbConnection,
        "SELECT * FROM connections WHERE uid = $1",
        uid,
    )
    .fetch_all(pool)
    .await?)
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

pub async fn get_connection_by_uid_and_username(
    uid: i32,
    username: &str,
    pool: &PgPool,
) -> Result<DbConnection> {
    Ok(sqlx::query_as!(
        DbConnection,
        "SELECT * FROM connections WHERE uid = $1 AND username = $2",
        uid,
        username,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update_connection_private_by_uid_and_username(
    uid: i32,
    username: &str,
    private: bool,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "UPDATE connections SET private = $3 WHERE uid = $1 AND username = $2",
        uid,
        username,
        private,
    )
    .execute(pool)
    .await?;

    Ok(())
}
