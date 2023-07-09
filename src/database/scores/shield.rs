use chrono::NaiveDateTime;
use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbScoreShield {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i64,
    pub shield: i32,
    pub region: String,
    pub timestamp: NaiveDateTime,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: NaiveDateTime,
}

impl AsRef<DbScoreShield> for DbScoreShield {
    fn as_ref(&self) -> &DbScoreShield {
        self
    }
}

pub async fn set_score_shield(score: &DbScoreShield, pool: &PgPool) -> Result<DbScoreShield> {
    sqlx::query!(
        "
        INSERT INTO
            scores_shield(uid, shield)
        VALUES
            ($1, $2)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            shield = EXCLUDED.shield
        ",
        score.uid,
        score.shield,
    )
    .execute(pool)
    .await?;

    get_score_shield_by_uid(score.uid, pool).await
}

pub async fn get_scores_shield(
    region: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbScoreShield>> {
    Ok(sqlx::query_as!(
        DbScoreShield,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY shield DESC) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY shield DESC) regional_rank,
                    *
                FROM
                    scores_shield
                NATURAL JOIN
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

pub async fn count_scores_shield(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!("SELECT COUNT(*) as count FROM scores_shield")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}

pub async fn get_score_shield_by_uid(uid: i64, pool: &PgPool) -> Result<DbScoreShield> {
    Ok(sqlx::query_as!(
        DbScoreShield,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY shield DESC) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY shield DESC) regional_rank,
                    *
                FROM
                    scores_shield
                NATURAL JOIN
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
