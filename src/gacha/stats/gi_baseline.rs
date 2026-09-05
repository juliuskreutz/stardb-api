//! Frozen pre-refactor scan baselines, extracted from 49daa02.
use super::*;
use crate::gacha::stats_math::average_or_zero;
pub(super) fn standard(wishes: &[Row], banners: &BannerCatalog) -> (f64,f64,f64,i32,i32) {
 let catalog = banners;
    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    for wish in &wishes {
        pull_4 += 1;
        pull_5 += 1;

        match wish.rarity.unwrap() {
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

(luck_4, luck_5, 0.0, 0, 0)
}
pub(super) fn character(wishes: &[Row], banners: &BannerCatalog) -> (f64,f64,f64,i32,i32) {
 let catalog = banners;
    let is_win = |item, timestamp| {
        banners
            .classify(
                PullPool::Gi(GiGachaType::Character),
                PullItem::Character(item),
                timestamp,
            )
             == crate::gacha::banner::BannerOutcome::Win
    };
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

    for wish in &wishes {
        pull_4 += 1;
        pull_5 += 1;

        match wish.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;

                let is_win = is_win(wish.character.unwrap(), wish.timestamp);
                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if is_win {
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

(luck_4, luck_5, win_rate, win_streak, loss_streak)
}
pub(super) fn weapon(wishes: &[Row], banners: &BannerCatalog) -> (f64,f64,f64,i32,i32) {
 let catalog = banners;
    let is_win = |item, timestamp| {
        banners
            .classify(
                PullPool::Gi(GiGachaType::Weapon),
                PullItem::Weapon(item),
                timestamp,
            )
             == crate::gacha::banner::BannerOutcome::Win
    };
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

    for wish in &wishes {
        pull_4 += 1;
        pull_5 += 1;

        match wish.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;

                let is_win = is_win(wish.weapon.unwrap(), wish.timestamp);
                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if is_win {
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

(luck_4, luck_5, win_rate, win_streak, loss_streak)
}
pub(super) fn chronicled(wishes: &[Row], banners: &BannerCatalog) -> (f64,f64,f64,i32,i32) {
 let catalog = banners;
    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    for wish in &wishes {
        pull_4 += 1;
        pull_5 += 1;

        match wish.rarity.unwrap() {
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

(luck_4, luck_5, 0.0, 0, 0)
}
#[derive(Clone)]
pub(super) struct Row { pub rarity: Option<i32>, pub character: Option<i32>, pub light_cone: Option<i32>, pub weapon: Option<i32>, pub w_engine: Option<i32>, pub timestamp: chrono::DateTime<chrono::Utc> }
