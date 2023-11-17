use anyhow::Result;
use sqlx::PgPool;

const THRESHOLD: i64 = 300;

pub struct DbUserAchievement {
    pub username: String,
    pub id: i64,
}

pub async fn add_user_achievement(
    user_achievement: &DbUserAchievement,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO users_achievements(username, id) VALUES($1, $2) ON CONFLICT(username, id) DO NOTHING",
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
                "DELETE FROM users_achievements WHERE username = $1 AND id = $2",
                user_achievement.username,
                related,
            )
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub async fn delete_user_achievement(
    user_achievement: &DbUserAchievement,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM users_achievements WHERE username = $1 AND id = $2",
        user_achievement.username,
        user_achievement.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_achievements_by_username(
    username: &str,
    pool: &PgPool,
) -> Result<Vec<DbUserAchievement>> {
    Ok(sqlx::query_as!(
        DbUserAchievement,
        "SELECT * FROM users_achievements WHERE username = $1",
        username
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_users_achievements_user_count(pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query!("SELECT COUNT(*) FROM users WHERE (SELECT COUNT(*) FROM users_achievements WHERE users_achievements.username = users.username) >= $1", THRESHOLD)
            .fetch_one(pool)
            .await?
            .count
            .unwrap_or_default(),
    )
}

pub struct DbAchievementUsersCount {
    pub id: i64,
    pub count: Option<i64>,
}

pub async fn get_achievements_users_count(pool: &PgPool) -> Result<Vec<DbAchievementUsersCount>> {
    Ok(sqlx::query_as!(
        DbAchievementUsersCount,
        "
        SELECT
            id,
            COUNT(*)
        FROM
            users_achievements
        JOIN
            (SELECT username FROM users_achievements GROUP BY username HAVING count(*) >= $1) counted_users
        ON
            users_achievements.username = counted_users.username
        GROUP BY
            id
        ",
        THRESHOLD,
    )
    .fetch_all(pool)
    .await?)
}
