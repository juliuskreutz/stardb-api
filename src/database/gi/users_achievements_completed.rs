use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserAchievementCompleted {
    pub username: String,
    pub id: i32,
}

pub async fn add(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    add_all(&user_achievement.username, &[user_achievement.id], pool).await
}
pub async fn add_all(username: &str, ids: &[i32], pool: &PgPool) -> Result<()> {
    crate::database::achievement_lists::add_all(
        crate::database::achievement_lists::Game::Gi,
        crate::database::achievement_lists::List::Completed,
        username,
        ids,
        pool,
    )
    .await
}
pub async fn delete_all(username: &str, ids: &[i32], pool: &PgPool) -> Result<()> {
    crate::database::achievement_lists::delete_all(
        crate::database::achievement_lists::Game::Gi,
        crate::database::achievement_lists::List::Completed,
        username,
        ids,
        pool,
    )
    .await
}

pub async fn delete(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    delete_all(&user_achievement.username, &[user_achievement.id], pool).await
}

pub async fn get_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserAchievementCompleted>> {
    Ok(sqlx::query_file_as!(
        DbUserAchievementCompleted,
        "sql/gi/users/achievements/completed/get_by_username.sql",
        username
    )
    .fetch_all(pool)
    .await?)
}
