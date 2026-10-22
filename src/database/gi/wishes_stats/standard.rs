//! Genshin per-UID pity and win-stat persistence; count reads feed population ranking. This module targets the standard pool.

use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use super::DbWishesStatCount;

pub struct DbWishesStatStandard {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

/// Upserts one UID’s calculated stats through the supplied executor.
/// The caller owns transaction commit/rollback; database failures propagate.
pub async fn set<'e, E>(stat: &DbWishesStatStandard, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/gi/wishes_stats/standard/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(executor)
    .await?;

    Ok(())
}

/// Returns this UID’s stored local stats, or None before recalculation.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWishesStatStandard>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatStandard,
        "sql/gi/wishes_stats/standard/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

/// Reads local stats joined with per-pool pull counts for population ranking.
/// SQL limits the population to UIDs with at least 100 pulls. Result order is unspecified.
pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWishesStatCount>> {
    Ok(sqlx::query_file_as!(
        DbWishesStatCount,
        "sql/gi/wishes_stats/standard/get_all.sql",
    )
    .fetch_all(pool)
    .await?)
}
