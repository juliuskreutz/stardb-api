//! ZZZ per-UID pity and win-stat persistence; count reads feed population ranking. This module targets the w engine pool.

use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use super::DbSignalsStatCount;

pub struct DbSignalsStatWEngine {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

/// Upserts one UID’s calculated stats through the supplied executor.
/// The caller owns transaction commit/rollback; database failures propagate.
pub async fn set<'e, E>(stat: &DbSignalsStatWEngine, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/zzz/signals_stats/w_engine/set.sql",
        stat.uid,
        stat.luck_a,
        stat.luck_s,
        stat.win_rate,
        stat.win_streak,
        stat.loss_streak,
    )
    .execute(executor)
    .await?;

    Ok(())
}

/// Returns this UID’s stored local stats, or None before recalculation.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatWEngine>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatWEngine,
        "sql/zzz/signals_stats/w_engine/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

/// Reads local stats joined with per-pool pull counts for population ranking.
/// Eligibility is applied by the updater; this read does not filter short histories. Result order is unspecified.
pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatCount>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatCount,
        "sql/zzz/signals_stats/w_engine/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
