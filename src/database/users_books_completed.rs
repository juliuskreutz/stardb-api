use anyhow::Result;
use sqlx::PgPool;

const THRESHOLD: i64 = 50;

pub struct DbUserBookCompleted {
    pub username: String,
    pub id: i32,
}

pub async fn add_user_book_completed(user_book: &DbUserBookCompleted, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users_books_completed(username, id) VALUES($1, $2)",
        user_book.username,
        user_book.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user_book_completed(
    user_book: &DbUserBookCompleted,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM users_books_completed WHERE username = $1 AND id = $2",
        user_book.username,
        user_book.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_books_completed_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserBookCompleted>> {
    Ok(sqlx::query_as!(
        DbUserBookCompleted,
        "SELECT * FROM users_books_completed WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_users_books_completed_user_count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query!("SELECT COUNT(*) FROM users WHERE (SELECT COUNT(*) FROM users_books_completed WHERE users_books_completed.username = users.username) >= $1", THRESHOLD)
            .fetch_one(pool)
            .await?
            .count
            .unwrap_or_default(),
    )
}

pub struct DbBookUsersCount {
    pub id: i32,
    pub count: Option<i64>,
}

pub async fn get_books_users_count(pool: &PgPool) -> Result<Vec<DbBookUsersCount>> {
    Ok(sqlx::query_as!(
        DbBookUsersCount,
        "
        SELECT
            id,
            COUNT(*)
        FROM
            users_books_completed
        JOIN
            (SELECT username FROM users_books_completed GROUP BY username HAVING count(*) >= $1) counted_users
        ON
            users_books_completed.username = counted_users.username
        GROUP BY
            id
        ",
        THRESHOLD,
    )
    .fetch_all(pool)
    .await?)
}
