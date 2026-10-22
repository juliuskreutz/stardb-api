//! HSR per-UID pity and win-stat persistence; count reads feed population ranking. This module targets the standard pool.

use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use crate::database::warps_stats::{DbWarpsStat, DbWarpsStatCount};

pub struct DbWarpsStatStandard {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

impl From<DbWarpsStatStandard> for DbWarpsStat {
    /// Adapts permanent-pool pity into the shared stat shape, with zeroed inapplicable win metrics.
    fn from(s: DbWarpsStatStandard) -> Self {
        DbWarpsStat {
            uid: s.uid,
            luck_4: s.luck_4,
            luck_5: s.luck_5,
            win_rate: 0.0,
            win_streak: 0,
            loss_streak: 0,
        }
    }
}

/// Upserts one UID’s calculated stats through the supplied executor.
/// The caller owns transaction commit/rollback; database failures propagate.
pub async fn set<'e, E>(stat: &DbWarpsStat, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/warps_stats/standard/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(executor)
    .await?;

    Ok(())
}

/// Reads local stats joined with per-pool pull counts for population ranking.
/// SQL limits the population to UIDs with at least 100 pulls. Result order is unspecified.
pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatCount>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatCount,
        "sql/warps_stats/standard/get_all_count.sql"
    )
    .fetch_all(pool)
    .await?)
}

/// Returns this UID’s stored local stats, or None before recalculation.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStat>> {
    let results = sqlx::query_file_as!(
        DbWarpsStatStandard,
        "sql/warps_stats/standard/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?;

    Ok(results.map(DbWarpsStat::from))
}
