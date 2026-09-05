use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserAchievementFavorite {
    pub username: String,
    pub id: i32,
}

pub async fn add(user_achievement: &DbUserAchievementFavorite, pool: &PgPool) -> Result<()> {
    add_all(&user_achievement.username, &[user_achievement.id], pool).await
}
pub async fn add_all(username: &str, ids: &[i32], pool: &PgPool) -> Result<()> {
    crate::database::achievement_lists::add_all(
        crate::database::achievement_lists::Game::Zzz,
        crate::database::achievement_lists::List::Favorites,
        username,
        ids,
        pool,
    )
    .await
}
pub async fn delete_all(username: &str, ids: &[i32], pool: &PgPool) -> Result<()> {
    crate::database::achievement_lists::delete_all(
        crate::database::achievement_lists::Game::Zzz,
        crate::database::achievement_lists::List::Favorites,
        username,
        ids,
        pool,
    )
    .await
}

pub async fn delete(user_achievement: &DbUserAchievementFavorite, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "DELETE FROM zzz_users_achievements_favorites WHERE username = $1 AND id = $2",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserAchievementFavorite>> {
    Ok(sqlx::query_as!(
        DbUserAchievementFavorite,
        "SELECT * FROM zzz_users_achievements_favorites WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}
