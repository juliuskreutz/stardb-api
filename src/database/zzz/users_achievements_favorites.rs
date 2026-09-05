use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserAchievementFavorite {
    pub username: String,
    pub id: i32,
}

/// Apply the transactional batch rules to one achievement, including alternate eviction.
pub async fn add(user_achievement: &DbUserAchievementFavorite, pool: &PgPool) -> Result<()> {
    add_all(&user_achievement.username, &[user_achievement.id], pool).await
}
/// Atomically add favorite IDs, retaining the last ID per set even when it is impossible.
/// Unknown IDs fail the whole batch before any list mutation.
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
/// Delete the requested IDs in one statement without disturbing other list entries.
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

/// Delete one achievement through the shared batch deletion path.
pub async fn delete(user_achievement: &DbUserAchievementFavorite, pool: &PgPool) -> Result<()> {
    delete_all(&user_achievement.username, &[user_achievement.id], pool).await
}

/// Fetch the user's stored achievement IDs; an empty list is a successful result.
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
