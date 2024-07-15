use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

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
    let uids = database::warps::get_uids(&pool).await?;

    info!("Starting standard");
    standard(&uids, &pool).await?;

    info!("Starting special");
    special(&uids, &pool).await?;

    info!("Starting lc");
    lc(&uids, &pool).await?;

    Ok(())
}

async fn standard(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let Some(warp_stat) = database::warps_stats_standard::get_by_uid(uid, pool).await? else {
            continue;
        };

        let count = database::warps::standard::get_count_by_uid(uid, pool).await? as i32;

        if count < 200 {
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
    sorted_luck_4.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

        database::warps_stats_standard::update_percentiles_by_uid(
            *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
            pool,
        )
        .await?;
    }

    Ok(())
}

async fn special(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let Some(warp_stat) = database::warps_stats_special::get_by_uid(uid, pool).await? else {
            continue;
        };

        let count = database::warps::special::get_count_by_uid(uid, pool).await? as i32;

        if count < 200 {
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
    sorted_luck_4.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

        database::warps_stats_special::update_percentiles_by_uid(
            *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
            pool,
        )
        .await?;
    }

    Ok(())
}

async fn lc(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let Some(warp_stat) = database::warps_stats_lc::get_by_uid(uid, pool).await? else {
            continue;
        };

        let count = database::warps::lc::get_count_by_uid(uid, pool).await? as i32;

        if count < 200 {
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
    sorted_luck_4.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

        database::warps_stats_lc::update_percentiles_by_uid(
            *uid,
            count_percentile,
            luck_4_percentile,
            luck_5_percentile,
            pool,
        )
        .await?;
    }

    Ok(())
}
