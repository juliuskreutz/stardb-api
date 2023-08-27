use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserBook {
    pub username: String,
    pub id: i64,
}

pub async fn add_user_book(user_book: &DbUserBook, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users_books(username, id) VALUES($1, $2)",
        user_book.username,
        user_book.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user_book(user_book: &DbUserBook, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "DELETE FROM users_books WHERE username = $1 AND id = $2",
        user_book.username,
        user_book.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_users_books(pool: &PgPool) -> Result<Vec<DbUserBook>> {
    Ok(sqlx::query_as!(DbUserBook, "SELECT * FROM users_books")
        .fetch_all(pool)
        .await?)
}

pub async fn get_user_books_by_username(username: &str, pool: &PgPool) -> Result<Vec<DbUserBook>> {
    Ok(sqlx::query_as!(
        DbUserBook,
        "SELECT * FROM users_books WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_users_books_user_count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query!("SELECT COUNT(*) FROM users WHERE (SELECT COUNT(*) FROM users_books WHERE users_books.username = users.username) >= 50")
            .fetch_one(pool)
            .await?
            .count
            .unwrap_or_default(),
    )
}
