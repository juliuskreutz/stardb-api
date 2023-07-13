use chrono::NaiveDateTime;
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
    pub video: String,
    pub region: String,
    pub timestamp: NaiveDateTime,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
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
            ($4::TEXT IS NULL OR LOWER(name) LIKE '%' || LOWER($4) || '%')
        ORDER BY
            (CASE WHEN $4 IS NOT NULL THEN LEVENSHTEIN(name, $4) ELSE global_rank END)
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

pub async fn count_scores_damage(region: &str, pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query!(
        "SELECT COUNT(*) as count FROM scores_damage NATURAL JOIN scores WHERE region = $1",
        region,
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap())
}

pub async fn get_score_damage_by_uid(
    uid: i64,
    character: Option<String>,
    support: Option<bool>,
    pool: &PgPool,
) -> Result<DbScoreDamage> {
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
                    ($2::TEXT IS NULL OR character = $2)
                AND
                    ($3::BOOLEAN IS NULL OR support = $3)
            ) ranked
        WHERE
            uid = $1
        ORDER BY
            global_rank
        LIMIT
            1
        ",
        uid,
        character,
        support,
    )
    .fetch_one(pool)
    .await?)
}
