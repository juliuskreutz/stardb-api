use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Default)]
pub struct DbScoreAchievement {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i32,
    pub region: String,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: DateTime<Utc>,
    pub timestamp: DateTime<Utc>,
}

pub async fn set(score: &DbScoreAchievement, pool: &PgPool) -> Result<DbScoreAchievement> {
    sqlx::query_as!(
        Score,
        "
        INSERT INTO
            scores_achievement(uid, timestamp)
        VALUES
            ($1, $2)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            timestamp = EXCLUDED.timestamp
        ",
        score.uid,
        score.timestamp,
    )
    .execute(pool)
    .await?;

    get_by_uid(score.uid, pool).await
}

pub async fn get(
    region: Option<&str>,
    query: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbScoreAchievement>> {
    Ok(sqlx::query_as!(
        DbScoreAchievement,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY achievement_count DESC, timestamp) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY achievement_count DESC, timestamp) regional_rank,
                    *
                FROM
                    scores_achievement
                NATURAL JOIN
                    mihomo
            ) ranked
        WHERE
            ($1::TEXT IS NULL OR region = $1)
        AND
            ($2::TEXT IS NULL OR LOWER(name) LIKE '%' || LOWER($2) || '%')
        ORDER BY
            (CASE WHEN $2 IS NOT NULL THEN LEVENSHTEIN(name, $2) ELSE global_rank END)
        LIMIT
            $3
        OFFSET
            $4
        ",
        region,
        query,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn count(region: Option<&str>, query: Option<&str>, pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!(
        "SELECT COUNT(*) as count FROM mihomo WHERE ($1::TEXT IS NULL OR region = $1) AND ($2::TEXT IS NULL OR LOWER(name) LIKE '%' || LOWER($2) || '%')",
        region,
        query,
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap())
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<DbScoreAchievement> {
    Ok(sqlx::query_as!(
        DbScoreAchievement,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY achievement_count DESC, timestamp) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY achievement_count DESC, timestamp) regional_rank,
                    *
                FROM
                    scores_achievement
                NATURAL JOIN
                    mihomo
            ) ranked
        WHERE
            uid = $1
        ",
        uid,
    )
    .fetch_one(pool)
    .await?)
}

#[derive(Default)]
pub struct DbScoreAchievementTimestamp {
    pub achievement_count: i32,
    pub timestamp: DateTime<Utc>,
}

pub async fn get_timestamp_by_uid(uid: i32, pool: &PgPool) -> Result<DbScoreAchievementTimestamp> {
    Ok(sqlx::query_as!(
        DbScoreAchievementTimestamp,
        "
        SELECT
            achievement_count,
            timestamp
        FROM
            scores_achievement
        NATURAL JOIN
            mihomo
        WHERE
            uid = $1
        ",
        uid,
    )
    .fetch_one(pool)
    .await?)
}
