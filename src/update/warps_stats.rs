use std::{
    cmp::Ordering,
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;
use tokio::time::timeout;

use crate::database;

pub async fn spawn(pool: PgPool) {
    actix::Arbiter::new().spawn(async move {
        let mut success = true;

        let mut interval = rt::time::interval(Duration::from_secs(60 * 60));

        loop {
            if success {
                interval.tick().await;
            }

            let start = Instant::now();

            // Set a 10-minute timeout for the update task
            match timeout(Duration::from_secs(10 * 60), update(pool.clone())).await {
                Ok(result) => {
                    match result {
                        Ok(_) => {
                            info!(
                                "Warps stats update succeeded in {}s",
                                start.elapsed().as_secs_f64()
                            );

                            success = true;
                        }
                        Err(e) => {
                            error!(
                                "Warps stats update failed with {e} in {}s",
                                start.elapsed().as_secs_f64()
                            );

                            success = false;
                        }
                    }
                }
                Err(_) => {
                    error!(
                        "Warps stats update timed out after {}s",
                        start.elapsed().as_secs_f64()
                    );

                    success = false;
                }
            }
        }
    });
}

async fn update(pool: PgPool) -> Result<()> {
    info!("Starting standard");
    standard(&pool).await?;

    info!("Starting special");
    special(&pool).await?;

    info!("Starting lc");
    lc(&pool).await?;

    info!("Starting collab");
    collab(&pool).await?;

    info!("Starting collab_lc");
    collab_lc(&pool).await?;

    Ok(())
}

async fn standard(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for warp_stat in database::warps_stats::standard::get_all(pool).await? {
        let uid = warp_stat.uid;

        let count = database::warps::standard::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, warp_stat.luck_4);
        luck_5_map.insert(uid, warp_stat.luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

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

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_4_percentile = luck_4_percentiles[uid] as f64 / len;
        let luck_5_percentile = luck_5_percentiles[uid] as f64 / len;

        let stat = database::warps_stats_global::standard::DbWarpsStatGlobalStandard {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::warps_stats_global::standard::set(&stat, pool).await?;
    }

    Ok(())
}

async fn special(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for warp_stat in database::warps_stats::special::get_all(pool).await? {
        let uid = warp_stat.uid;

        let count = database::warps::special::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, warp_stat.luck_4);
        luck_5_map.insert(uid, warp_stat.luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

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

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_4_percentile = luck_4_percentiles[uid] as f64 / len;
        let luck_5_percentile = luck_5_percentiles[uid] as f64 / len;

        let stat = database::warps_stats_global::special::DbWarpsStatGlobalSpecial {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::warps_stats_global::special::set(&stat, pool).await?;
    }

    Ok(())
}

async fn lc(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for warp_stat in database::warps_stats::lc::get_all(pool).await? {
        let uid = warp_stat.uid;

        let count = database::warps::lc::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, warp_stat.luck_4);
        luck_5_map.insert(uid, warp_stat.luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

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

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_4_percentile = luck_4_percentiles[uid] as f64 / len;
        let luck_5_percentile = luck_5_percentiles[uid] as f64 / len;

        let stat = database::warps_stats_global::lc::DbWarpsStatGlobalLc {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::warps_stats_global::lc::set(&stat, pool).await?;
    }

    Ok(())
}

async fn collab(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for warp_stat in database::warps_stats::collab::get_all(pool).await? {
        let uid = warp_stat.uid;

        let count = database::warps::collab::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, warp_stat.luck_4);
        luck_5_map.insert(uid, warp_stat.luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

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

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_4_percentile = luck_4_percentiles[uid] as f64 / len;
        let luck_5_percentile = luck_5_percentiles[uid] as f64 / len;

        let stat = database::warps_stats_global::collab::DbWarpsStatGlobalCollab {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::warps_stats_global::collab::set(&stat, pool).await?;
    }

    Ok(())
}

async fn collab_lc(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for warp_stat in database::warps_stats::collab_lc::get_all(pool).await? {
        let uid = warp_stat.uid;

        let count = database::warps::collab_lc::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, warp_stat.luck_4);
        luck_5_map.insert(uid, warp_stat.luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

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

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_4_percentile = luck_4_percentiles[uid] as f64 / len;
        let luck_5_percentile = luck_5_percentiles[uid] as f64 / len;

        let stat = database::warps_stats_global::collab_lc::DbWarpsStatGlobalCollabLc {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::warps_stats_global::collab_lc::set(&stat, pool).await?;
    }

    Ok(())
}