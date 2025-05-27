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
                                "Wishes stats update succeeded in {}s",
                                start.elapsed().as_secs_f64()
                            );

                            success = true;
                        }
                        Err(e) => {
                            error!(
                                "Wishes stats update failed with {e} in {}s",
                                start.elapsed().as_secs_f64()
                            );

                            success = false;
                        }
                    }
                }
                Err(_) => {
                    error!(
                        "Wishes stats update timed out after {}s",
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

    info!("Starting character");
    character(&pool).await?;

    info!("Starting weapon");
    weapon(&pool).await?;

    info!("Starting chronicled");
    chronicled(&pool).await?;

    Ok(())
}

async fn standard(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for wish_stat in database::gi::wishes_stats::standard::get_all(pool).await? {
        let uid = wish_stat.uid;

        let count = database::gi::wishes::standard::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, wish_stat.luck_4);
        luck_5_map.insert(uid, wish_stat.luck_5);
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

        let stat = database::gi::wishes_stats_global::standard::DbWishesStatGlobalStandard {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::gi::wishes_stats_global::standard::set(&stat, pool).await?;
    }

    Ok(())
}

async fn character(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for wish_stat in database::gi::wishes_stats::character::get_all(pool).await? {
        let uid = wish_stat.uid;

        let count = database::gi::wishes::character::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, wish_stat.luck_4);
        luck_5_map.insert(uid, wish_stat.luck_5);
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

        let stat = database::gi::wishes_stats_global::character::DbWishesStatGlobalCharacter {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::gi::wishes_stats_global::character::set(&stat, pool).await?;
    }

    Ok(())
}

async fn weapon(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for wish_stat in database::gi::wishes_stats::weapon::get_all(pool).await? {
        let uid = wish_stat.uid;

        let count = database::gi::wishes::weapon::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, wish_stat.luck_4);
        luck_5_map.insert(uid, wish_stat.luck_5);
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

        let stat = database::gi::wishes_stats_global::weapon::DbWishesStatGlobalWeapon {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::gi::wishes_stats_global::weapon::set(&stat, pool).await?;
    }

    Ok(())
}

async fn chronicled(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for wish_stat in database::gi::wishes_stats::chronicled::get_all(pool).await? {
        let uid = wish_stat.uid;

        let count = database::gi::wishes::chronicled::get_count_by_uid(uid, pool).await? as i32;

        if count < 100 {
            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_4_map.insert(uid, wish_stat.luck_4);
        luck_5_map.insert(uid, wish_stat.luck_5);
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

        let stat = database::gi::wishes_stats_global::chronicled::DbWishesStatGlobalChronicled {
            uid: *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
        };

        database::gi::wishes_stats_global::chronicled::set(&stat, pool).await?;
    }

    Ok(())
}
