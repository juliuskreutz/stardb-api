use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserAchievementFavorite {
    pub username: String,
    pub id: i32,
}

pub async fn add_user_achievement_favorite(
    user_achievement: &DbUserAchievementFavorite,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users_achievements_favorites(username, id) VALUES($1, $2) ON CONFLICT(username, id) DO NOTHING",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    if let Some(set) = sqlx::query!(
        "SELECT set FROM achievements WHERE id = $1",
        user_achievement.id,
    )
    .fetch_one(pool)
    .await?
    .set
    {
        for related in super::get_related(user_achievement.id, set, pool).await? {
            sqlx::query!(
                "DELETE FROM users_achievements_favorites WHERE username = $1 AND id = $2",
                user_achievement.username,
                related,
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub async fn delete_user_achievement_favorite(
    user_achievement: &DbUserAchievementFavorite,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM users_achievements_favorites WHERE username = $1 AND id = $2",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_achievements_favorites_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserAchievementFavorite>> {
    Ok(sqlx::query_as!(
        DbUserAchievementFavorite,
        "SELECT * FROM users_achievements_favorites WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}
