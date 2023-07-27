mod damage;
mod heal;
mod shield;

pub use damage::*;
pub use heal::*;
pub use shield::*;

use chrono::NaiveDateTime;
use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbScore {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i64,
    pub region: String,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub timestamp: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn set_score(score: &DbScore, pool: &PgPool) -> Result<DbScore> {
    sqlx::query_as!(
        Score,
        "
        INSERT INTO
            scores(uid, region, name, level, signature, avatar_icon, achievement_count, timestamp)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            name = EXCLUDED.name,
            level = EXCLUDED.level,
            signature = EXCLUDED.signature,
            avatar_icon = EXCLUDED.avatar_icon,
            achievement_count = EXCLUDED.achievement_count,
            timestamp = EXCLUDED.timestamp,
            updated_at = now()
        ",
        score.uid,
        score.region,
        score.name,
        score.level,
        score.signature,
        score.avatar_icon,
        score.achievement_count,
        score.timestamp,
    )
    .execute(pool)
    .await?;

    get_score_by_uid(score.uid, pool).await
}

pub async fn get_scores(
    region: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbScore>> {
    Ok(sqlx::query_as!(
        DbScore,
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
                    scores
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

pub async fn count_scores(region: &str, pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!(
        "SELECT COUNT(*) as count FROM scores WHERE region = $1",
        region
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap())
}

pub async fn get_score_by_uid(uid: i64, pool: &PgPool) -> Result<DbScore> {
    Ok(sqlx::query_as!(
        DbScore,
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
                    scores
            ) ranked
        WHERE
            uid = $1
        ",
        uid,
    )
    .fetch_one(pool)
    .await?)
}
