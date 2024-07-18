use anyhow::Result;
use sqlx::PgPool;

pub struct DbUser {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

pub async fn set(user: &DbUser, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/users/set.sql",
        user.username,
        user.password,
        user.email,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_one_by_username(username: &str, pool: &PgPool) -> Result<DbUser> {
    Ok(
        sqlx::query_file_as!(DbUser, "sql/users/get_one_by_username.sql", username)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn update_email_by_username(username: &str, email: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/users/update_email_by_username.sql", username, email,)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_password_by_username(
    username: &str,
    password: &str,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/users/update_password_by_username.sql",
        username,
        password,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_email_by_username(username: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!("sql/users/delete_email_by_username.sql", username)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn count_emails(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_file!("sql/users/count_emails.sql")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}
