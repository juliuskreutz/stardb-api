use anyhow::Result;
use sqlx::PgPool;

pub struct DbConnection {
    pub uid: i32,
    pub username: String,
    pub verified: bool,
    pub private: bool,
}

pub async fn set(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/connections/set.sql",
        connection.uid,
        connection.username,
        connection.verified,
        false,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/connections/delete.sql",
        connection.uid,
        connection.username,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Vec<DbConnection>> {
    Ok(
        sqlx::query_file_as!(DbConnection, "sql/zzz/connections/get_by_uid.sql", uid)
            .fetch_all(pool)
            .await?,
    )
}

pub async fn get_by_username(username: &str, pool: &PgPool) -> Result<Vec<DbConnection>> {
    Ok(sqlx::query_file_as!(
        DbConnection,
        "sql/zzz/connections/get_by_username.sql",
        username,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_uid_and_username(
    uid: i32,
    username: &str,
    pool: &PgPool,
) -> Result<DbConnection> {
    Ok(sqlx::query_file_as!(
        DbConnection,
        "sql/zzz/connections/get_by_uid_and_username.sql",
        uid,
        username,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update_private_by_uid_and_username(
    uid: i32,
    username: &str,
    private: bool,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/connections/update_private_by_uid_and_username.sql",
        uid,
        username,
        private,
    )
    .execute(pool)
    .await?;

    Ok(())
}
