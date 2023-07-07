use chrono::NaiveDateTime;
use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbScoreHeal {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i64,
    pub heal: i32,
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

impl AsRef<DbScoreHeal> for DbScoreHeal {
    fn as_ref(&self) -> &DbScoreHeal {
        self
    }
}

pub async fn set_score_heal(score: &DbScoreHeal, pool: &PgPool) -> Result<DbScoreHeal> {
    sqlx::query!(
        "
        INSERT INTO
            scores_heal(uid, heal)
        VALUES
            ($1, $2)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            heal = EXCLUDED.heal
        ",
        score.uid,
        score.heal,
    )
    .execute(pool)
    .await?;

    get_score_heal_by_uid(score.uid, pool).await
}

pub async fn get_scores_heal(
    region: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbScoreHeal>> {
    Ok(sqlx::query_as!(
        DbScoreHeal,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY heal DESC) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY heal DESC) regional_rank,
                    *
                FROM
                    scores_heal
                NATURAL JOIN
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

pub async fn count_scores_heal(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!("SELECT COUNT(*) as count FROM scores_heal")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}

pub async fn get_score_heal_by_uid(uid: i64, pool: &PgPool) -> Result<DbScoreHeal> {
    Ok(sqlx::query_as!(
        DbScoreHeal,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY heal DESC) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY heal DESC) regional_rank,
                    *
                FROM
                    scores_heal
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
