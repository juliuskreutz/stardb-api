use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbScoreDamage {
    pub global_rank: Option<i64>,
    pub regional_rank: Option<i64>,
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub region: String,
    pub timestamp: NaiveDateTime,
    pub info: Value,
    pub updated_at: NaiveDateTime,
}

impl AsRef<DbScoreDamage> for DbScoreDamage {
    fn as_ref(&self) -> &DbScoreDamage {
        self
    }
}

pub async fn set_score_damage(score: &DbScoreDamage, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            scores_damage(uid, character, support, damage)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (uid, character, support)
        DO UPDATE SET
            damage = EXCLUDED.damage
        ",
        score.uid,
        score.character,
        score.support,
        score.damage,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_scores_damage(
    character: Option<String>,
    support: Option<bool>,
    region: Option<String>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    pool: &PgPool,
) -> Result<Vec<DbScoreDamage>> {
    Ok(sqlx::query_as!(
        DbScoreDamage,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY damage DESC) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY damage DESC) regional_rank,
                    *
                FROM
                    scores_damage
                NATURAL JOIN
                    scores
                WHERE
                    ($1::TEXT IS NULL OR character = $1)
                AND
                    ($2::BOOLEAN IS NULL OR support = $2)
            ) ranked
        WHERE
            ($3::TEXT IS NULL OR region = $3)
        AND
            ($4::TEXT IS NULL OR LOWER(info -> 'player' ->> 'nickname') LIKE '%' || LOWER($4) || '%')
        ORDER BY
            (CASE WHEN $4 IS NOT NULL THEN LEVENSHTEIN(info -> 'player' ->> 'nickname', $4) ELSE global_rank END)
        LIMIT
            $5
        OFFSET
            $6
        ",
        character,
        support,
        region,
        query,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn count_scores_damage(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!("SELECT COUNT(*) as count FROM scores_damage")
        .fetch_one(pool)
        .await?
        .count
        .unwrap())
}

pub async fn get_scores_damage_by_uid(uid: i64, pool: &PgPool) -> Result<Vec<DbScoreDamage>> {
    Ok(sqlx::query_as!(
        DbScoreDamage,
        "
        SELECT
            *
        FROM
            (
                SELECT
                    RANK() OVER (ORDER BY damage DESC) global_rank,
                    RANK() OVER (PARTITION BY region ORDER BY damage DESC) regional_rank,
                    *
                FROM
                    scores_damage
                NATURAL JOIN
                    scores
            ) ranked
        WHERE
            uid = $1
        ",
        uid,
    )
    .fetch_all(pool)
    .await?)
}
