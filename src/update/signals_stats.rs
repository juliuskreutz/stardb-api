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
                                "Signals stats update succeeded in {}s",
                                start.elapsed().as_secs_f64()
                            );

                            success = true;
                        }
                        Err(e) => {
                            error!(
                                "Signals stats update failed with {e} in {}s",
                                start.elapsed().as_secs_f64()
                            );

                            success = false;
                        }
                    }
                }
                Err(_) => {
                    error!(
                        "Signals stats update timed out after {}s",
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

    info!("Starting w_engine");
    w_engine(&pool).await?;

    info!("Starting bangboo");
    bangboo(&pool).await?;

    Ok(())
}

async fn standard(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for signal_stat in database::zzz::signals_stats::standard::get_all(pool).await? {
        let uid = signal_stat.uid;

        let count = database::zzz::signals::standard::get_count_by_uid(uid, pool).await? as i32;

        if count < 50 || signal_stat.luck_s == 0.0 {
            database::zzz::signals_stats_global::standard::delete_by_uid(uid, pool).await?;

            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_a_map.insert(uid, signal_stat.luck_a);
        luck_s_map.insert(uid, signal_stat.luck_s);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_a: Vec<(i32, f64)> = luck_a_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_a
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let count_percentiles: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_a_percentiles: HashMap<_, _> = sorted_luck_a
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_s_percentiles: HashMap<_, _> = sorted_luck_s
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_a_percentile = luck_a_percentiles[uid] as f64 / len;
        let luck_s_percentile = luck_s_percentiles[uid] as f64 / len;

        let stat = database::zzz::signals_stats_global::standard::DbSignalsStatGlobalStandard {
            uid: *uid,
            count_percentile,
            luck_a_percentile,
            luck_s_percentile,
        };

        database::zzz::signals_stats_global::standard::set(&stat, pool).await?;
    }

    Ok(())
}

async fn special(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for signal_stat in database::zzz::signals_stats::special::get_all(pool).await? {
        let uid = signal_stat.uid;

        let count = database::zzz::signals::special::get_count_by_uid(uid, pool).await? as i32;

        if count < 50 || signal_stat.luck_s == 0.0 {
            database::zzz::signals_stats_global::special::delete_by_uid(uid, pool).await?;

            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_a_map.insert(uid, signal_stat.luck_a);
        luck_s_map.insert(uid, signal_stat.luck_s);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_a: Vec<(i32, f64)> = luck_a_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_a
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let count_percentiles: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_a_percentiles: HashMap<_, _> = sorted_luck_a
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_s_percentiles: HashMap<_, _> = sorted_luck_s
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_a_percentile = luck_a_percentiles[uid] as f64 / len;
        let luck_s_percentile = luck_s_percentiles[uid] as f64 / len;

        let stat = database::zzz::signals_stats_global::special::DbSignalsStatGlobalSpecial {
            uid: *uid,
            count_percentile,
            luck_a_percentile,
            luck_s_percentile,
        };

        database::zzz::signals_stats_global::special::set(&stat, pool).await?;
    }

    Ok(())
}

async fn w_engine(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for signal_stat in database::zzz::signals_stats::w_engine::get_all(pool).await? {
        let uid = signal_stat.uid;

        let count = database::zzz::signals::w_engine::get_count_by_uid(uid, pool).await? as i32;

        if count < 50 || signal_stat.luck_s == 0.0 {
            database::zzz::signals_stats_global::w_engine::delete_by_uid(uid, pool).await?;

            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_a_map.insert(uid, signal_stat.luck_a);
        luck_s_map.insert(uid, signal_stat.luck_s);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_a: Vec<(i32, f64)> = luck_a_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_a
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let count_percentiles: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_a_percentiles: HashMap<_, _> = sorted_luck_a
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_s_percentiles: HashMap<_, _> = sorted_luck_s
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_a_percentile = luck_a_percentiles[uid] as f64 / len;
        let luck_s_percentile = luck_s_percentiles[uid] as f64 / len;

        let stat = database::zzz::signals_stats_global::w_engine::DbSignalsStatGlobalWEngine {
            uid: *uid,
            count_percentile,
            luck_a_percentile,
            luck_s_percentile,
        };

        database::zzz::signals_stats_global::w_engine::set(&stat, pool).await?;
    }

    Ok(())
}

async fn bangboo(pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for signal_stat in database::zzz::signals_stats::bangboo::get_all(pool).await? {
        let uid = signal_stat.uid;

        let count = database::zzz::signals::bangboo::get_count_by_uid(uid, pool).await? as i32;

        if count < 50 || signal_stat.luck_s == 0.0 {
            database::zzz::signals_stats_global::bangboo::delete_by_uid(uid, pool).await?;

            continue;
        }

        stat_uids.push(uid);

        count_map.insert(uid, count);
        luck_a_map.insert(uid, signal_stat.luck_a);
        luck_s_map.insert(uid, signal_stat.luck_s);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_a: Vec<(i32, f64)> = luck_a_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_a
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s
        .sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap_or(Ordering::Equal));

    let count_percentiles: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_a_percentiles: HashMap<_, _> = sorted_luck_a
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_s_percentiles: HashMap<_, _> = sorted_luck_s
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let len = stat_uids.len() as f64;
    for uid in &stat_uids {
        let count_percentile = count_percentiles[uid] as f64 / len;
        let luck_a_percentile = luck_a_percentiles[uid] as f64 / len;
        let luck_s_percentile = luck_s_percentiles[uid] as f64 / len;

        let stat = database::zzz::signals_stats_global::bangboo::DbSignalsStatGlobalBangboo {
            uid: *uid,
            count_percentile,
            luck_a_percentile,
            luck_s_percentile,
        };

        database::zzz::signals_stats_global::bangboo::set(&stat, pool).await?;
    }

    Ok(())
}
