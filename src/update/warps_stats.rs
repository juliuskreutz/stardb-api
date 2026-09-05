use std::time::{Duration, Instant};

use anyhow::Result;
use sqlx::PgPool;

use crate::{
    database,
    database::warps_stats_global::DbWarpsStatGlobal,
    gacha::global_stats::{calculate_percentiles, PercentileInput, UPDATE_BATCH_SIZE},
};

pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "warps_stats",
        Duration::from_secs(3600),
        Duration::from_secs(30),
        move || update(pool.clone()),
    );
}

async fn update(pool: PgPool) -> Result<()> {
    standard(&pool).await?;
    special(&pool).await?;
    lc(&pool).await?;
    collab(&pool).await?;
    collab_lc(&pool).await?;

    Ok(())
}

async fn standard(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Standard, pool).await
}

async fn special(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Special, pool).await
}

async fn lc(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Lc, pool).await
}

async fn collab(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::Collab, pool).await
}

async fn collab_lc(pool: &PgPool) -> Result<()> {
    refresh(crate::GachaType::CollabLc, pool).await
}

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
