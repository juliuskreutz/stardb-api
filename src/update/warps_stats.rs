//! Refreshes HSR population percentiles on the shared periodic job driver.
//! Each pool is updated in batches; a failure leaves completed batches for the next retry.

use std::time::{Duration, Instant};

use anyhow::Result;
use sqlx::PgPool;

use crate::{
    database,
    database::warps_stats_global::DbWarpsStatGlobal,
    gacha::global_stats::{calculate_percentiles, PercentileInput, UPDATE_BATCH_SIZE},
};

/// Starts the hourly percentile job with a 30-second retry delay on failure.
/// Scheduling is delegated to the shared driver; this call does not await job completion.
pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "warps_stats",
        Duration::from_secs(3600),
        Duration::from_secs(30),
        move || update(pool.clone()),
    );
}

/// Refreshes supported pools sequentially, stopping at the first failure.
/// Previously written batches remain committed for the next idempotent retry.
async fn update(pool: PgPool) -> Result<()> {
    standard(&pool).await?;
    special(&pool).await?;
    lc(&pool).await?;
    collab(&pool).await?;
    collab_lc(&pool).await?;

    Ok(())
}

/// Refreshes the standard pool through the shared within-game batch path.
async fn standard(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Standard, pool).await
}

/// Refreshes the special pool through the shared within-game batch path.
async fn special(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Special, pool).await
}

/// Refreshes the lc pool through the shared within-game batch path.
async fn lc(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Lc, pool).await
}

/// Refreshes the collab pool through the shared within-game batch path.
async fn collab(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Collab, pool).await
}

/// Refreshes the collab lc pool through the shared within-game batch path.
async fn collab_lc(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::CollabLc, pool).await
}

/// Converts count-joined local stats into population percentiles for bulk persistence.
fn calculate_stats(
    banner_type: &str,
    stats: Vec<database::warps_stats::DbWarpsStatCount>,
) -> Vec<DbWarpsStatGlobal> {
    info!(
        "Calculating {} warp stats for banner: {}",
        stats.len(),
        banner_type
    );
    calculate_percentiles(
        stats
            .into_iter()
            .map(|stat| PercentileInput {
                uid: stat.uid,
                count: stat.warp_count.unwrap_or(0) as i32,
                luck_low: stat.luck_4,
                luck_high: stat.luck_5,
            })
            .collect(),
    )
    .into_iter()
    .map(|stat| DbWarpsStatGlobal {
        uid: stat.uid,
        count_percentile: stat.count,
        luck_4_percentile: stat.luck_low,
        luck_5_percentile: stat.luck_high,
    })
    .collect()
}

/// Fetches one pool’s population and writes calculated percentiles in bounded batches.
/// Eligibility comes from the count query. Writes are separately committed,
/// so failures may leave earlier batches refreshed until the next retry.
async fn refresh(kind: crate::GachaType, pool: &PgPool) -> Result<()> {
    info!("Starting {kind} warps stats update");
    let start = Instant::now();

    let warp_stats = database::warps_stats::get_all_by_pool(kind, pool).await?;
    if warp_stats.is_empty() {
        info!("No {kind} warps stats to update");
        return Ok(());
    }

    let stats = calculate_stats(&kind.to_string(), warp_stats);
    let total_batches = stats.len().div_ceil(UPDATE_BATCH_SIZE);
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!(
            "processing batch {} of {} for banner {}",
            i + 1,
            total_batches,
            kind.to_string()
        );
        database::warps_stats_global::set_bulk_by_pool(kind, batch, pool).await?;
    }

    info!(
        "{kind} warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}
