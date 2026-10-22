//! Global percentile persistence for W-Engine Reverberation signals.

use anyhow::Result;
use sqlx::PgPool;

use super::DbSignalsStatGlobal;

/// Upserts a batch of global percentile rows in one round trip.
pub async fn set_bulk(stats: &[DbSignalsStatGlobal], pool: &PgPool) -> Result<()> {
    if stats.is_empty() {
        return Ok(());
    }
    let uids: Vec<_> = stats.iter().map(|stat| stat.uid).collect();
    let count: Vec<_> = stats.iter().map(|stat| stat.count_percentile).collect();
    let luck_a: Vec<_> = stats.iter().map(|stat| stat.luck_a_percentile).collect();
    let luck_s: Vec<_> = stats.iter().map(|stat| stat.luck_s_percentile).collect();

    sqlx::query_file!(
        "sql/zzz/signals_stats_global/w_engine_reverberation/set_bulk.sql",
        &uids,
        &count,
        &luck_a,
        &luck_s,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Fetches one UID's stored percentile row; eligibility is enforced by refresh jobs.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbSignalsStatGlobal>> {
    Ok(sqlx::query_file_as!(
        DbSignalsStatGlobal,
        "sql/zzz/signals_stats_global/w_engine_reverberation/get_by_uid.sql",
        uid
    )
    .fetch_optional(pool)
    .await?)
}

/// Deletes a batch of ineligible global rows in one round trip.
pub async fn delete_bulk(uids: &[i32], pool: &PgPool) -> Result<()> {
    if uids.is_empty() {
        return Ok(());
    }
    sqlx::query_file!(
        "sql/zzz/signals_stats_global/w_engine_reverberation/delete_bulk.sql",
        uids
    )
    .execute(pool)
    .await?;

    Ok(())
}
