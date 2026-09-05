use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserAchievementCompleted {
    pub username: String,
    pub id: i32,
}

/// Apply the transactional batch rules to one achievement, including alternate eviction.
pub async fn add(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    add_all(&user_achievement.username, &[user_achievement.id], pool).await
}
/// Atomically add completed IDs, ignoring impossible entries and retaining the last eligible ID per set.
/// Unknown IDs fail the whole batch before any list mutation.
pub async fn add_all(username: &str, ids: &[i32], pool: &PgPool) -> Result<()> {
    crate::database::achievement_lists::add_all(
        crate::database::achievement_lists::Game::Hsr,
        crate::database::achievement_lists::List::Completed,
        username,
        ids,
        pool,
    )
    .await
}
/// Delete the requested IDs in one statement without disturbing other list entries.
pub async fn delete_all(username: &str, ids: &[i32], pool: &PgPool) -> Result<()> {
    crate::database::achievement_lists::delete_all(
        crate::database::achievement_lists::Game::Hsr,
        crate::database::achievement_lists::List::Completed,
        username,
        ids,
        pool,
    )
    .await
}

/// Delete one achievement through the shared batch deletion path.
pub async fn delete(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    delete_all(&user_achievement.username, &[user_achievement.id], pool).await
}

/// Fetch the user's stored achievement IDs; an empty list is a successful result.
pub async fn get_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserAchievementCompleted>> {
    Ok(sqlx::query_as!(
        DbUserAchievementCompleted,
        "SELECT * FROM users_achievements_completed WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}

/// Count users with at least one stored completion for the tracker population display.
pub async fn user_count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query!("SELECT COUNT(*) FROM users WHERE EXISTS (SELECT * FROM users_achievements_completed WHERE users.username = users_achievements_completed.username)")
            .fetch_one(pool)
            .await?
            .count
            .unwrap_or_default(),
    )
}
