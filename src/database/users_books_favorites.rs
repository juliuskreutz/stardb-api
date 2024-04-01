use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserBookFavorite {
    pub username: String,
    pub id: i64,
}

pub async fn add_user_book_favorite(user_book: &DbUserBookFavorite, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users_books_favorites(username, id) VALUES($1, $2) ON CONFLICT(username, id) DO NOTHING",
        user_book.username,
        user_book.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user_book_favorite(
    user_book: &DbUserBookFavorite,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM users_books_favorites WHERE username = $1 AND id = $2",
        user_book.username,
        user_book.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_books_favorites_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserBookFavorite>> {
    Ok(sqlx::query_as!(
        DbUserBookFavorite,
        "SELECT * FROM users_books_favorites WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}
