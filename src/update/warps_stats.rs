use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::{database, GachaType};

pub async fn spawn(pool: PgPool) {
    actix::Arbiter::new().spawn(async move {
        let mut success = true;

        let mut interval = rt::time::interval(Duration::from_secs(60 * 60 * 24));

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
    let uids = database::get_warp_uids(&pool).await?;

    //info!("Starting standard");
    //standard(&uids, &pool).await?;
    //
    //info!("Starting special");
    //special(&uids, &pool).await?;

    info!("Starting lc");
    lc(&uids, &pool).await?;

    Ok(())
}

async fn standard(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut set_all = database::warps_stats_standard::SetAll::default();

    for &uid in uids {
        let warps =
            database::get_warp_infos_by_uid_and_gacha_type(uid, GachaType::Standard, pool).await?;

        let count = warps.len() as i32;

        let mut pull_4 = 0;
        let mut sum_4 = 0;
        let mut count_4 = 0;

        let mut pull_5 = 0;
        let mut sum_5 = 0;
        let mut count_5 = 0;

        for warp in &warps {
            pull_4 += 1;
            pull_5 += 1;

            match warp.rarity.unwrap() {
                4 => {
                    count_4 += 1;
                    sum_4 += pull_4;
                    pull_4 = 0;
                }
                5 => {
                    count_5 += 1;
                    sum_5 += pull_5;
                    pull_5 = 0;
                }
                _ => {}
            }
        }

        if count_5 < 3 {
            continue;
        }

        let luck_4 = sum_4 as f64 / count_4 as f64;
        let luck_5 = sum_5 as f64 / count_5 as f64;

        set_all.uid.push(uid);

        set_all.count.push(count);
        count_map.insert(uid, count);

        set_all.luck_4.push(luck_4);
        luck_4_map.insert(uid, luck_4);

        set_all.luck_5.push(luck_5);
        luck_5_map.insert(uid, luck_5);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let count_ranks: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_4_ranks: HashMap<_, _> = sorted_luck_4
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_5_ranks: HashMap<_, _> = sorted_luck_5
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    for uid in &set_all.uid {
        let count_rank = count_ranks[uid] as i32;
        let luck4_rank = luck_4_ranks[uid] as i32;
        let luck5_rank = luck_5_ranks[uid] as i32;

        set_all.count_rank.push(count_rank);
        set_all.luck_4_rank.push(luck4_rank);
        set_all.luck_5_rank.push(luck5_rank);
    }

    database::warps_stats_standard::set_all(&set_all, pool).await?;

    Ok(())
}

async fn special(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut set_all = database::warps_stats_special::SetAll::default();

    for &uid in uids {
        let warps =
            database::get_warp_infos_by_uid_and_gacha_type(uid, GachaType::Special, pool).await?;

        let count = warps.len() as i32;

        let mut pull_4 = 0;
        let mut sum_4 = 0;
        let mut count_4 = 0;

        let mut pull_5 = 0;
        let mut sum_5 = 0;
        let mut count_5 = 0;

        let mut guarantee = false;

        let mut sum_win = 0;
        let mut count_win = 0;

        let mut win_streak = 0;
        let mut max_win_streak = 0;

        let mut loss_streak = 0;
        let mut max_loss_streak = 0;

        for warp in &warps {
            pull_4 += 1;
            pull_5 += 1;

            match warp.rarity.unwrap() {
                4 => {
                    count_4 += 1;
                    sum_4 += pull_4;
                    pull_4 = 0;
                }
                5 => {
                    count_5 += 1;
                    sum_5 += pull_5;
                    pull_5 = 0;

                    if guarantee {
                        guarantee = false;
                    } else {
                        count_win += 1;

                        if [1209, 1004, 1101, 1211, 1104, 1107, 1003]
                            .contains(&warp.character.unwrap())
                        {
                            max_win_streak = max_win_streak.max(win_streak);
                            win_streak = 0;

                            loss_streak += 1;

                            guarantee = true;
                        } else {
                            sum_win += 1;

                            max_loss_streak = max_loss_streak.max(loss_streak);
                            loss_streak = 0;

                            win_streak += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        if count_5 < 5 {
            continue;
        }

        let luck_4 = sum_4 as f64 / count_4 as f64;
        let luck_5 = sum_5 as f64 / count_5 as f64;
        let win_rate = sum_win as f64 / count_win as f64;

        set_all.uid.push(uid);

        set_all.count.push(count);
        count_map.insert(uid, count);

        set_all.luck_4.push(luck_4);
        luck_4_map.insert(uid, luck_4);

        set_all.luck_5.push(luck_5);
        luck_5_map.insert(uid, luck_5);

        set_all.win_rate.push(win_rate);
        set_all.win_streak.push(max_win_streak);
        set_all.loss_streak.push(max_loss_streak);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let count_ranks: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_4_ranks: HashMap<_, _> = sorted_luck_4
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_5_ranks: HashMap<_, _> = sorted_luck_5
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    for uid in &set_all.uid {
        let count_rank = count_ranks[uid] as i32;
        let luck4_rank = luck_4_ranks[uid] as i32;
        let luck5_rank = luck_5_ranks[uid] as i32;

        set_all.count_rank.push(count_rank);
        set_all.luck_4_rank.push(luck4_rank);
        set_all.luck_5_rank.push(luck5_rank);
    }

    database::warps_stats_special::set_all(&set_all, pool).await?;

    Ok(())
}

async fn lc(uids: &[i32], pool: &PgPool) -> Result<()> {
    let mut count_map = HashMap::new();
    let mut luck_4_map = HashMap::new();
    let mut luck_5_map = HashMap::new();

    let mut set_all = database::warps_stats_lc::SetAll::default();

    for &uid in uids {
        let warps =
            database::get_warp_infos_by_uid_and_gacha_type(uid, GachaType::Lc, pool).await?;

        let count = warps.len() as i32;

        let mut pull_4 = 0;
        let mut sum_4 = 0;
        let mut count_4 = 0;

        let mut pull_5 = 0;
        let mut sum_5 = 0;
        let mut count_5 = 0;

        let mut guarantee = false;

        let mut sum_win = 0;
        let mut count_win = 0;

        let mut win_streak = 0;
        let mut max_win_streak = 0;

        let mut loss_streak = 0;
        let mut max_loss_streak = 0;

        for warp in &warps {
            pull_4 += 1;
            pull_5 += 1;

            match warp.rarity.unwrap() {
                4 => {
                    count_4 += 1;
                    sum_4 += pull_4;
                    pull_4 = 0;
                }
                5 => {
                    count_5 += 1;
                    sum_5 += pull_5;
                    pull_5 = 0;

                    if guarantee {
                        guarantee = false;
                    } else {
                        count_win += 1;

                        if [23000, 23002, 23003, 23004, 23005, 23012, 23013]
                            .contains(&warp.light_cone.unwrap())
                        {
                            sum_win += 1;

                            max_win_streak = max_win_streak.max(win_streak);
                            win_streak = 0;

                            loss_streak += 1;

                            guarantee = true;
                        } else {
                            max_loss_streak = max_loss_streak.max(loss_streak);
                            loss_streak = 0;

                            win_streak += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        if count_5 < 5 {
            continue;
        }

        let luck_4 = sum_4 as f64 / count_4 as f64;
        let luck_5 = sum_5 as f64 / count_5 as f64;
        let win_rate = sum_win as f64 / count_win as f64;

        set_all.uid.push(uid);

        set_all.count.push(count);
        count_map.insert(uid, count);

        set_all.luck_4.push(luck_4);
        luck_4_map.insert(uid, luck_4);

        set_all.luck_5.push(luck_5);
        luck_5_map.insert(uid, luck_5);

        set_all.win_rate.push(win_rate);
        set_all.win_streak.push(max_win_streak);
        set_all.loss_streak.push(max_loss_streak);
    }

    let mut sorted_count: Vec<(i32, i32)> = count_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_count.sort_unstable_by(|(_, v1), (_, v2)| v2.cmp(v1));

    let mut sorted_luck_4: Vec<(i32, f64)> = luck_4_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_4.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let mut sorted_luck_5: Vec<(i32, f64)> = luck_5_map.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_luck_5.sort_unstable_by(|(_, v1), (_, v2)| v1.partial_cmp(v2).unwrap());

    let count_ranks: HashMap<_, _> = sorted_count
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_4_ranks: HashMap<_, _> = sorted_luck_4
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    let luck_5_ranks: HashMap<_, _> = sorted_luck_5
        .into_iter()
        .enumerate()
        .map(|(i, (uid, _))| (uid, i))
        .collect();

    for uid in &set_all.uid {
        let count_rank = count_ranks[uid] as i32;
        let luck4_rank = luck_4_ranks[uid] as i32;
        let luck5_rank = luck_5_ranks[uid] as i32;

        set_all.count_rank.push(count_rank);
        set_all.luck_4_rank.push(luck4_rank);
        set_all.luck_5_rank.push(luck5_rank);
    }

    database::warps_stats_lc::set_all(&set_all, pool).await?;

    Ok(())
}
