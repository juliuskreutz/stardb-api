//! Global percentile persistence for W-Engine Reverberation signals.

use anyhow::Result;
use sqlx::PgPool;

/// Population-relative ranks displayed beside one UID's local stats.
pub struct DbSignalsStatGlobalWEngineReverberation {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_a_percentile: f64,
    pub luck_s_percentile: f64,
}

/// Upserts one UID's current percentile row.
pub async fn set(stat: &DbSignalsStatGlobalWEngineReverberation, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/w_engine_reverberation/set.sql",
        stat.uid,
        stat.count_percentile,
        stat.luck_a_percentile,
        stat.luck_s_percentile,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetches one UID's percentile row, if the history is eligible.
pub async fn get_by_uid(
    uid: i32,
    pool: &PgPool,
) -> Result<Option<DbSignalsStatGlobalWEngineReverberation>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatGlobalWEngineReverberation,
        "sql/zzz/signals_stats_global/w_engine_reverberation/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

/// Removes a percentile row when its underlying history is no longer eligible.
pub async fn delete_by_uid(uid: i32, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/w_engine_reverberation/delete_by_uid.sql",
        uid
    )
    .execute(pool)
    .await?;

    Ok(())
}
