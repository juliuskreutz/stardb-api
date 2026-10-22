//! Per-user statistics persistence for Exclusive Rescreening signals.

use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use super::DbSignalsStatCount;

/// Persisted pity and confirmed win/loss metrics for one UID.
pub struct DbSignalsStatExclusiveRescreening {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}

/// Upserts one stat row on a caller-owned executor.
pub async fn set<'e, E>(stat: &DbSignalsStatExclusiveRescreening, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/zzz/signals_stats/exclusive_rescreening/set.sql",
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

/// Fetches one UID's tracker stats, if calculated.
pub async fn get_by_uid(
    uid: i32,
    pool: &PgPool,
) -> Result<Option<DbSignalsStatExclusiveRescreening>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatExclusiveRescreening,
        "sql/zzz/signals_stats/exclusive_rescreening/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?)
}

/// Lists all per-user rows used by the global percentile updater.
pub async fn get_all(pool: &PgPool) -> Result<Vec<DbSignalsStatCount>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatCount,
        "sql/zzz/signals_stats/exclusive_rescreening/get_all.sql"
    )
    .fetch_all(pool)
    .await?)
}
