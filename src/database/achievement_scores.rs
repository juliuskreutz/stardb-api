//! Achievement ranks are read models; timestamp writes avoid the expensive ranked join.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Default)]
pub struct DbScoreAchievement {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i32,
    pub region: String,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: DateTime<Utc>,
}

pub struct DbScoreAchievementWrite {
    pub uid: i32,
    pub timestamp: DateTime<Utc>,
}

/// Upsert the score timestamp without running a ranked read; profile fields remain in mihomo.
pub async fn set(score: &DbScoreAchievementWrite, pool: &PgPool) -> Result<()> {
    sqlx::query!("INSERT INTO scores_achievement(uid, timestamp) VALUES ($1, $2) ON CONFLICT (uid) DO UPDATE SET timestamp = EXCLUDED.timestamp", score.uid, score.timestamp)
        .execute(pool).await?;
    Ok(())
}

/// Select refresh candidates without computing display ranks or loading profile fields.
/// UID breaks exact score/timestamp ties deterministically for maintenance pagination;
/// public ranks still treat those rows as peers. Concurrent edits can still shift offsets.
pub async fn get_refresh_uids(limit: i64, offset: i64, pool: &PgPool) -> Result<Vec<i32>> {
    Ok(sqlx::query_scalar::<_, i32>(
        "SELECT s.uid FROM scores_achievement s JOIN mihomo m ON m.uid = s.uid
         ORDER BY m.achievement_count DESC, s.timestamp ASC, s.uid ASC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?)
}

/// Rank the complete score population before applying region/name filters and pagination.
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
            global_rank AS \"global_rank!\", regional_rank AS \"regional_rank!\", uid, region, name, level, signature, avatar_icon, achievement_count, updated_at
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

/// Count mihomo profiles matching the region/name filters used by score pagination.
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

/// Read one UID with ranks calculated across the whole population; return None when absent.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbScoreAchievement>> {
    Ok(sqlx::query_as!(
        DbScoreAchievement,
        "
        SELECT
            global_rank AS \"global_rank!\", regional_rank AS \"regional_rank!\", uid, region, name, level, signature, avatar_icon, achievement_count, updated_at
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
    .fetch_optional(pool)
    .await?)
}

#[derive(Default)]
pub struct DbScoreAchievementTimestamp {
    pub achievement_count: i32,
    pub timestamp: DateTime<Utc>,
}

/// Read the stored achievement count and tie-breaking timestamp; an absent joined row is an error.
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

#[cfg(test)]
mod refresh_tests {
    use super::*;

    #[sqlx::test]
    async fn sql_performance_refresh_pages_match_rank_order(pool: PgPool) {
        // Mix regions and leave every tenth profile without a score.
        sqlx::query("INSERT INTO mihomo(uid, region, name, level, signature, avatar_icon, achievement_count)
            SELECT i, CASE WHEN i % 2 = 0 THEN 'na' ELSE 'eu' END, 'seed', 1, '', '', 10000-i
            FROM generate_series(1, 1000) i").execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO scores_achievement(uid, timestamp)
            SELECT uid, '2026-01-01'::timestamptz FROM mihomo WHERE uid % 10 != 0",
        )
        .execute(&pool)
        .await
        .unwrap();
        for offset in [0, 100, 800, 900, 1100] {
            let old: Vec<_> = get(None, None, Some(100), Some(offset), &pool)
                .await
                .unwrap()
                .into_iter()
                .map(|s| s.uid)
                .collect();
            assert_eq!(get_refresh_uids(100, offset, &pool).await.unwrap(), old);
        }
        // Exact peers now have a defined UID order for refresh only.
        sqlx::query("UPDATE mihomo SET achievement_count = 20000 WHERE uid IN (1, 2, 3)")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(get_refresh_uids(2, 0, &pool).await.unwrap(), vec![1, 2]);
        assert_eq!(get_refresh_uids(1, 2, &pool).await.unwrap(), vec![3]);
        for uid in [1, 2, 3] {
            assert_eq!(
                get_by_uid(uid, &pool).await.unwrap().unwrap().global_rank,
                1
            );
        }
    }
}
