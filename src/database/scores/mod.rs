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
    pub avatar_icon: String,
    pub signature: String,
    pub character_count: i32,
    pub achievement_count: i32,
    pub character_name: String,
    pub character_icon: String,
    pub path_icon: String,
    pub element_color: String,
    pub element_icon: String,
    pub timestamp: NaiveDateTime,
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
            scores(uid, region, name, level, avatar_icon, signature, character_count, achievement_count, character_name, character_icon, path_icon, element_color, element_icon, timestamp)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            name = EXCLUDED.name,
            level = EXCLUDED.level,
            avatar_icon = EXCLUDED.avatar_icon,
            signature = EXCLUDED.signature,
            character_count = EXCLUDED.character_count,
            achievement_count = EXCLUDED.achievement_count,
            character_name = EXCLUDED.character_name,
            character_icon = EXCLUDED.character_icon,
            path_icon = EXCLUDED.path_icon,
            element_color = EXCLUDED.element_color,
            element_icon = EXCLUDED.element_icon,
            timestamp = EXCLUDED.timestamp
        ",
        score.uid,
        score.region,
        score.name,
        score.level,
        score.avatar_icon,
        score.signature,
        score.character_count,
        score.achievement_count,
        score.character_name,
        score.character_icon,
        score.path_icon,
        score.element_color,
        score.element_icon,
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
            ($2::TEXT IS NULL OR uid::TEXT = $2 OR LOWER(name) LIKE '%' || LOWER($2) || '%')
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
