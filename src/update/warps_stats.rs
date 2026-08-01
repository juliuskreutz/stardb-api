use std::time::{Duration, Instant};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::{
    database,
    database::warps_stats_global::DbWarpsStatGlobal,
    gacha::global_stats::{calculate_percentiles, PercentileInput, UPDATE_BATCH_SIZE},
};

pub async fn spawn(pool: PgPool) {
    actix::Arbiter::new().spawn(async move {
        let mut success = true;

        let mut interval = rt::time::interval(Duration::from_secs(60 * 60));

        loop {
            if success {
                interval.tick().await;
            }

            let start = Instant::now();

            if let Err(e) = update(pool.clone()).await {
                error!(
                    "Warps stats update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );

                success = false;
            } else {
                info!(
                    "Warps stats update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );

                success = true;
            }
        }
    });
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
    info!("Starting standard warps stats update");
    let start = Instant::now();

    let warp_stats = database::warps_stats::standard::get_all(pool).await?;
    if warp_stats.is_empty() {
        info!("No standard warps stats to update");
        return Ok(());
    }

    let stats = calculate_stats("standard", warp_stats);
    let total_batches = stats.len().div_ceil(UPDATE_BATCH_SIZE);
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!(
            "processing batch {} of {} for banner {}",
            i + 1,
            total_batches,
            "standard"
        );
        database::warps_stats_global::standard::set_bulk(batch, pool).await?;
    }

    info!(
        "Standard warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );
    Ok(())
}

async fn special(pool: &PgPool) -> Result<()> {
    info!("Starting special warps stats update");
    let start = Instant::now();

    let warp_stats = database::warps_stats::special::get_all(pool).await?;
    if warp_stats.is_empty() {
        info!("No special warps stats to update");
        return Ok(());
    }

    let stats = calculate_stats("special", warp_stats);
    let total_batches = stats.len().div_ceil(UPDATE_BATCH_SIZE);
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!(
            "processing batch {} of {} for banner {}",
            i + 1,
            total_batches,
            "special"
        );
        database::warps_stats_global::special::set_bulk(batch, pool).await?;
    }

    info!(
        "Special warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

async fn lc(pool: &PgPool) -> Result<()> {
    info!("Starting lc warps stats update");
    let start = Instant::now();

    let warp_stats = database::warps_stats::lc::get_all(pool).await?;
    if warp_stats.is_empty() {
        info!("No lc warps stats to update");
        return Ok(());
    }

    let stats = calculate_stats("lc", warp_stats);
    let total_batches = stats.len().div_ceil(UPDATE_BATCH_SIZE);
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!(
            "processing batch {} of {} for banner {}",
            i + 1,
            total_batches,
            "lc"
        );
        database::warps_stats_global::lc::set_bulk(batch, pool).await?;
    }

    info!(
        "LC warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

async fn collab(pool: &PgPool) -> Result<()> {
    info!("Starting collab warps stats update");
    let start = Instant::now();

    let warp_stats = database::warps_stats::collab::get_all(pool).await?;
    if warp_stats.is_empty() {
        info!("No collab warps stats to update");
        return Ok(());
    }

    let stats = calculate_stats("collab", warp_stats);
    let total_batches = stats.len().div_ceil(UPDATE_BATCH_SIZE);
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!(
            "processing batch {} of {} for banner {}",
            i + 1,
            total_batches,
            "collab"
        );
        database::warps_stats_global::collab::set_bulk(batch, pool).await?;
    }

    info!(
        "Collab warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

async fn collab_lc(pool: &PgPool) -> Result<()> {
    info!("Starting collab lc warps stats update");
    let start = Instant::now();

    let warp_stats = database::warps_stats::collab_lc::get_all(pool).await?;
    if warp_stats.is_empty() {
        info!("No collab lc warps stats to update");
        return Ok(());
    }

    let stats = calculate_stats("collab lc", warp_stats);
    let total_batches = stats.len().div_ceil(UPDATE_BATCH_SIZE);
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!(
            "processing batch {} of {} for banner {}",
            i + 1,
            total_batches,
            "collab lc"
        );
        database::warps_stats_global::collab_lc::set_bulk(batch, pool).await?;
    }

    info!(
        "Collab LC warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(())
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
