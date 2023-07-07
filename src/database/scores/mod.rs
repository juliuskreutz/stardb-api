mod damage;
mod heal;
mod shield;

pub use damage::*;
pub use heal::*;
pub use shield::*;

use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbScore {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i64,
    pub region: String,
    pub timestamp: NaiveDateTime,
    pub info: Value,
    pub updated_at: NaiveDateTime,
}

impl AsRef<DbScore> for DbScore {
    fn as_ref(&self) -> &DbScore {
        self
    }
}

pub async fn set_score(score: &DbScore, pool: &PgPool) -> Result<DbScore> {
    sqlx::query_as!(
        Score,
        "
        INSERT INTO
            scores(uid, region, timestamp, info)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            timestamp = EXCLUDED.timestamp,
            info = EXCLUDED.info,
            updated_at = now()
        ",
        score.uid,
        score.region,
        score.timestamp,
        score.info
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
                    RANK() OVER (ORDER BY info -> 'player' -> 'space_info' ->> 'achievement_count' DESC, timestamp) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY info -> 'player' -> 'space_info' ->> 'achievement_count' DESC, timestamp) regional_rank,
                    *
                FROM
                    scores
            ) ranked
        WHERE
            ($1::TEXT IS NULL OR region = $1)
        AND
            ($2::TEXT IS NULL OR LOWER(info -> 'player' ->> 'nickname') LIKE '%' || LOWER($2) || '%')
        ORDER BY
            (CASE WHEN $2 IS NOT NULL THEN LEVENSHTEIN(info -> 'player' ->> 'nickname', $2) ELSE global_rank END)
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

pub async fn count_scores(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!("SELECT COUNT(*) as count FROM scores")
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
                    RANK() OVER (ORDER BY info -> 'player' -> 'space_info' ->> 'achievement_count' DESC, timestamp) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY info -> 'player' -> 'space_info' ->> 'achievement_count' DESC, timestamp) regional_rank,
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
