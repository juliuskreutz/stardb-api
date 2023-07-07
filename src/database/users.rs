use sqlx::PgPool;

use crate::{api::users::User, Result};

pub async fn set_user(user: &User, pool: &PgPool) -> Result<()> {
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

pub async fn get_user_by_username(username: &str, pool: &PgPool) -> Result<User> {
    Ok(
        sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn is_admin(username: &str, pool: &PgPool) -> bool {
    sqlx::query!("SELECT * FROM admins WHERE username = $1", username)
        .fetch_one(pool)
        .await
        .is_ok()
}
