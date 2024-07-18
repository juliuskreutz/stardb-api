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
                    "Signals stats update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );

                success = false;
            } else {
                info!(
                    "Signals stats update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );

                success = true;
            }
        }
    });
}

async fn update(pool: PgPool) -> Result<()> {
    let uids = database::zzz::signals::get_uids(&pool).await?;

    for &uid in uids {
        calculate_stats_standard(uid, &pool).await?;
    }

    info!("Starting standard");
    standard(&uids, &pool).await?;

    info!("Starting special");
    special(&uids, &pool).await?;

    info!("Starting w_engine");
    w_engine(&uids, &pool).await?;

    info!("Starting bangboo");
    bangboo(&uids, &pool).await?;

    Ok(())
}

async fn standard(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let signal_stat = database::zzz::signals_stats::standard::get_by_uid(uid, pool).await?;

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
    sorted_luck_a.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

async fn special(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let signal_stat = database::zzz::signals_stats::special::get_by_uid(uid, pool).await?;

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
    sorted_luck_a.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

async fn w_engine(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let signal_stat = database::zzz::signals_stats::w_engine::get_by_uid(uid, pool).await?;

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
    sorted_luck_a.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

async fn bangboo(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_a_map = HashMap::new();
    let mut luck_s_map = HashMap::new();

    let mut stat_uids = Vec::new();

    for &uid in uids {
        let signal_stat = database::zzz::signals_stats::bangboo::get_by_uid(uid, pool).await?;

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
    sorted_luck_a.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_s: Vec<(i32, f64)> = luck_s_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_s.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

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

async fn calculate_stats_standard(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::standard::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut first_s_rank = true;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                if first_s_rank {
                    first_s_rank = false;
                    pull_s = 0;
                    continue;
                }
                
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;
            }
            _ => {}
        }
    }

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };

    let stat = database::zzz::signals_stats::standard::DbSignalsStatStandard {
        uid,
        luck_a,
        luck_s,
    };
    database::zzz::signals_stats::standard::set(&stat, pool).await?;

    Ok(())
}
