use anyhow::Result;
use sqlx::PgPool;

use crate::database::gi::achievements::DbAchievement;

pub struct DbUserAchievementFavorite {
    pub username: String,
    pub id: i32,
}

pub async fn add(user_achievement: &DbUserAchievementFavorite, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/users/achievements/favorites/set.sql",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    if let Some(set) = sqlx::query_file_as!(
        DbAchievement,
        "sql/gi/achievements/get_one_by_id.sql",
        user_achievement.id,
        "en",
    )
    .fetch_one(pool)
    .await?
    .set
    {
        for related in
            super::achievements::get_all_related_ids(user_achievement.id, set, pool).await?
        {
            sqlx::query_file!(
                "sql/gi/users/achievements/favorites/delete.sql",
                user_achievement.username,
                related,
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub async fn delete(user_achievement: &DbUserAchievementFavorite, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/users/achievements/favorites/delete.sql",
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
    Ok(sqlx::query_file_as!(
        DbUserAchievementFavorite,
        "sql/gi/users/achievements/favorites/get_by_username.sql",
        username,
    )
    .fetch_all(pool)
    .await?)
}
