use anyhow::Result;
use sqlx::PgPool;

pub struct DbUserAchievementCompleted {
    pub username: String,
    pub id: i32,
}

pub async fn add(user_achievement: &DbUserAchievementCompleted, pool: &PgPool) -> Result<()> {
    if sqlx::query!(
        "SELECT impossible FROM zzz_achievements WHERE id = $1",
        user_achievement.id
    )
    .fetch_one(pool)
    .await?
    .impossible
    {
        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO zzz_users_achievements_completed(username, id) VALUES($1, $2) ON CONFLICT(username, id) DO NOTHING",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    if let Some(set) = sqlx::query!(
        "SELECT set FROM zzz_achievements WHERE id = $1",
        user_achievement.id,
    )
    .fetch_one(pool)
    .await?
    .set
    {
        for related in
            super::achievements::get_all_related_ids(user_achievement.id, set, pool).await?
        {
            sqlx::query!(
                "DELETE FROM zzz_users_achievements_completed WHERE username = $1 AND id = $2",
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
    sqlx::query!(
        "DELETE FROM zzz_users_achievements_completed WHERE username = $1 AND id = $2",
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
) -> Result<Vec<DbUserAchievementCompleted>> {
    Ok(sqlx::query_as!(
        DbUserAchievementCompleted,
        "SELECT * FROM zzz_users_achievements_completed WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_user_count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query!("SELECT COUNT(*) FROM users WHERE EXISTS (SELECT * FROM zzz_users_achievements_completed WHERE users.username = zzz_users_achievements_completed.username)")
            .fetch_one(pool)
            .await?
            .count
            .unwrap_or_default(),
    )
}
