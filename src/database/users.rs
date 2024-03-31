use anyhow::Result;
use sqlx::PgPool;

pub struct DbUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

pub async fn set(user: &DbUser, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users(username, password, email) VALUES($1, $2, $3)",
        user.username,
        user.password,
        user.email,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_username(username: &str, pool: &PgPool) -> Result<DbUser> {
    Ok(
        sqlx::query_as!(DbUser, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn update_email(username: &str, email: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET email = $2 WHERE username = $1",
        username,
        email,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_password(username: &str, password: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET password = $2 WHERE username = $1",
        username,
        password,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_email(username: &str, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE users SET email = NULL WHERE username = $1",
        username
    )
    .execute(pool)
    .await?;

    Ok(())
}
