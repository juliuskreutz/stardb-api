use anyhow::Result;
use sqlx::PgPool;

use crate::database::gi::achievements::DbAchievement;

pub struct DbUserAchievementCompleted {
    pub username: String,
    pub id: i32,
}

pub async fn add(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    if sqlx::query_file!(
        "sql/gi/achievements/get_one_by_id.sql",
        user_achievement.id,
        "en"
    )
    .fetch_one(pool)
    .await?
    .impossible
    {
        return Ok(());
    }

    sqlx::query_file!(
        "sql/gi/users/achievements/completed/set.sql",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    if let Some(set) = sqlx::query_file_as!(
        DbAchievement,
        "sql/gi/achievements/get_one_by_id.sql",
        user_achievement.id,
        "en"
    )
    .fetch_one(pool)
    .await?
    .set
    {
        for related in
            super::achievements::get_all_related_ids(user_achievement.id, set, pool).await?
        {
            sqlx::query_file!(
                "sql/gi/users/achievements/completed/delete.sql",
                user_achievement.username,
                related,
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub async fn delete(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/users/achievements/completed/delete.sql",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_by_username(username: &str, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/users/achievements/completed/delete_by_username.sql",
        username,
    )
    .execute(pool)
    .await?;

    Ok(())
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

pub async fn count_users(threshhold: i64, pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_file!(
        "sql/gi/users/achievements/completed/count_users.sql",
        threshhold
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap())
}
