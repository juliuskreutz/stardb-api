use sqlx::PgPool;

use crate::Result;

pub struct DbUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub admin: bool,
}

pub async fn set_user(user: &DbUser, pool: &PgPool) -> Result<()> {
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

pub async fn get_user_by_username(username: &str, pool: &PgPool) -> Result<DbUser> {
    Ok(
        sqlx::query_as!(DbUser, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}
