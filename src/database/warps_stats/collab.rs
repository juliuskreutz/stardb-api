//! HSR per-UID pity and win-stat persistence; count reads feed population ranking. This module targets the collab pool.

use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use crate::database::warps_stats::{DbWarpsStat, DbWarpsStatCount};

/// Upserts one UID’s calculated stats through the supplied executor.
/// The caller owns transaction commit/rollback; database failures propagate.
pub async fn set<'e, E>(stat: &DbWarpsStat, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/warps_stats/collab/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
        stat.win_rate,
        stat.win_streak,
        stat.loss_streak,
    )
    .execute(executor)
    .await?;

    Ok(())
}

/// Reads local stats joined with per-pool pull counts for population ranking.
/// SQL limits the population to UIDs with at least 100 pulls. Result order is unspecified.
pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatCount>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStatCount, "sql/warps_stats/collab/get_all_count.sql")
            .fetch_all(pool)
            .await?,
    )
}

/// Returns this UID’s stored local stats, or None before recalculation.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStat>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStat, "sql/warps_stats/collab/get_by_uid.sql", uid,)
            .fetch_optional(pool)
            .await?,
    )
}
