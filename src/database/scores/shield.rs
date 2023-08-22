use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

#[derive(Default)]
pub struct DbScoreShield {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i64,
    pub shield: i32,
    pub video: String,
    pub region: String,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: NaiveDateTime,
}

pub async fn set_score_shield(score: &DbScoreShield, pool: &PgPool) -> Result<DbScoreShield> {
    sqlx::query!(
        "
        INSERT INTO
            scores_shield(uid, shield, video)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            shield = EXCLUDED.shield,
            video = EXCLUDED.video
        ",
        score.uid,
        score.shield,
        score.video,
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

pub async fn count_scores_shield(
    region: Option<&str>,
    query: Option<&str>,
    pool: &PgPool,
) -> Result<i64> {
    Ok(sqlx::query!(
        "SELECT COUNT(*) as count FROM scores_shield NATURAL JOIN mihomo WHERE ($1::TEXT IS NULL OR region = $1) AND ($2::TEXT IS NULL OR LOWER(name) LIKE '%' || LOWER($2) || '%')",
        region,
        query,
    )
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
