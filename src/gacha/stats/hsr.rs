use std::collections::HashMap;
use std::ops::Range;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::{
    api::banner_helpers::{self, HSR_STANDARD},
    database,
    gacha::stats_math::average_or_zero,
};

pub(crate) async fn recalculate_hsr_uid(
    uid: i32,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let banners = load_banners(&mut *connection).await?;
    calculate_stats_standard(uid, &mut *connection).await?;
    calculate_stats_special(uid, &banners, &mut *connection).await?;
    calculate_stats_lc(uid, &banners, &mut *connection).await?;
    calculate_stats_collab(uid, &banners, &mut *connection).await?;
    calculate_stats_collab_lc(uid, &banners, &mut *connection).await?;
    Ok(())
}

type BannerMap = HashMap<i32, Vec<Range<DateTime<Utc>>>>;

async fn load_banners(connection: &mut PgConnection) -> anyhow::Result<BannerMap> {
    let mut banners: BannerMap = HashMap::new();
    for banner in database::banners::get_all_with_executor(&mut *connection).await? {
        if let Some(character) = banner.character {
            banners
                .entry(character)
                .or_default()
                .push(banner.start..banner.end);
        }
        if let Some(light_cone) = banner.light_cone {
            banners
                .entry(light_cone)
                .or_default()
                .push(banner.start..banner.end);
        }
    }
    Ok(banners)
}

async fn calculate_stats_standard(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let warps = database::warps::standard::get_infos_by_uid(uid, &mut *connection).await?;

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

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        // these don't apply to standard warps
        win_rate: 0.0,
        win_streak: 0,
        loss_streak: 0,
    };
    database::warps_stats::standard::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_special(
    uid: i32,
    banners: &BannerMap,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let is_win = banner_helpers::is_win_fn(banners, HSR_STANDARD);

    let warps = database::warps::special::get_infos_by_uid(uid, &mut *connection).await?;

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

                    if is_win(warp.character.unwrap(), warp.timestamp) {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);

                        continue;
                    }

                    win_streak = 0;

                    loss_streak += 1;
                    max_loss_streak = max_loss_streak.max(loss_streak);

                    guarantee = true;
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::special::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_lc(
    uid: i32,
    banners: &BannerMap,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let is_win = banner_helpers::is_win_fn(banners, HSR_STANDARD);

    let warps = database::warps::lc::get_infos_by_uid(uid, &mut *connection).await?;

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

                    if is_win(warp.light_cone.unwrap(), warp.timestamp) {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);

                        continue;
                    }

                    win_streak = 0;

                    loss_streak += 1;
                    max_loss_streak = max_loss_streak.max(loss_streak);

                    guarantee = true;
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::lc::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_collab(
    uid: i32,
    banners: &BannerMap,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let is_win = banner_helpers::is_win_fn(banners, HSR_STANDARD);

    let warps = database::warps::collab::get_infos_by_uid(uid, &mut *connection).await?;

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

                    if is_win(warp.character.unwrap(), warp.timestamp) {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);

                        continue;
                    }

                    win_streak = 0;

                    loss_streak += 1;
                    max_loss_streak = max_loss_streak.max(loss_streak);

                    guarantee = true;
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::collab::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_collab_lc(
    uid: i32,
    banners: &BannerMap,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let is_win = banner_helpers::is_win_fn(banners, HSR_STANDARD);

    let warps = database::warps::collab_lc::get_infos_by_uid(uid, &mut *connection).await?;

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

                    if is_win(warp.light_cone.unwrap(), warp.timestamp) {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);

                        continue;
                    }

                    win_streak = 0;

                    loss_streak += 1;
                    max_loss_streak = max_loss_streak.max(loss_streak);

                    guarantee = true;
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::collab_lc::set(&stat, &mut *connection).await?;

    Ok(())
}
