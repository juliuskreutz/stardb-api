use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::database;
use crate::database::warps_stats_global::DbWarpsStatGlobal;

const UPDATE_BATCH_SIZE: usize = 1000;

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
    if let Err(e) = standard(&pool).await {
        error!("standard warps stats update failed: {e}");
    }

    if let Err(e) = special(&pool).await {
        error!("special warps stats update failed: {e}");
    }

    if let Err(e) = lc(&pool).await {
        error!("lc warps stats update failed: {e}");
    }

    if let Err(e) = collab(&pool).await {
        error!("collab warps stats update failed: {e}");
    }

    if let Err(e) = collab_lc(&pool).await {
        error!("collab_lc warps stats update failed: {e}");
    }

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
    let total_batches = (stats.len() + UPDATE_BATCH_SIZE - 1) / UPDATE_BATCH_SIZE;
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!("processing batch {} of {} for banner {}", i + 1, total_batches, "standard");
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
    let total_batches = (stats.len() + UPDATE_BATCH_SIZE - 1) / UPDATE_BATCH_SIZE;
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!("processing batch {} of {} for banner {}", i + 1, total_batches, "special");
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
    let total_batches = (stats.len() + UPDATE_BATCH_SIZE - 1) / UPDATE_BATCH_SIZE;
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!("processing batch {} of {} for banner {}", i + 1, total_batches, "lc");
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
    let total_batches = (stats.len() + UPDATE_BATCH_SIZE - 1) / UPDATE_BATCH_SIZE;
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!("processing batch {} of {} for banner {}", i + 1, total_batches, "collab");
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
    let total_batches = (stats.len() + UPDATE_BATCH_SIZE - 1) / UPDATE_BATCH_SIZE;
    for (i, batch) in stats.chunks(UPDATE_BATCH_SIZE).enumerate() {
        info!("processing batch {} of {} for banner {}", i + 1, total_batches, "collab lc");
        database::warps_stats_global::collab_lc::set_bulk(batch, pool).await?;
    }

    info!(
        "Collab LC warps stats updated: {} in {}s",
        stats.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

fn calculate_stats(banner_type: &str, stats: Vec<database::warps_stats::DbWarpsStatCount>) -> Vec<DbWarpsStatGlobal> {
    info!("Calculating {} warp stats for banner: {}", stats.len(), banner_type);
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();
    let mut stat_uids = Vec::new();

    for warp_stat in stats {
        let uid = warp_stat.uid;
        let count = warp_stat.warp_count.unwrap_or(0) as i32;

        stat_uids.push(uid);
        count_map.insert(uid, count);
        luck_4_map.insert(uid, warp_stat.luck_4);
        luck_5_map.insert(uid, warp_stat.luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    // Use total_cmp to get a total order for f64 (handles NaN deterministically)
    sorted_luck_4
        .sort_unstable_by(|(_, v1), (_, v2)| v1.total_cmp(v2));

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    // Use total_cmp to get a total order for f64 (handles NaN deterministically)
    sorted_luck_5
        .sort_unstable_by(|(_, v1), (_, v2)| v1.total_cmp(v2));

    let count_percentiles: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_4_percentiles: HashMap<_, _> = sorted_luck_4
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_5_percentiles: HashMap<_, _> = sorted_luck_5
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let mut stats = Vec::new();
    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_4_percentile = luck_4_percentiles[uid] as f64 / len;
        let luck_5_percentile = luck_5_percentiles[uid] as f64 / len;

        let stat = DbWarpsStatGlobal {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        stats.push(stat);
    }

    stats
}
