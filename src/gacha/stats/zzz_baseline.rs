#![allow(unused_variables, dead_code)]
//! Frozen pre-refactor scan baselines, extracted from 49daa02.
use super::*;
use crate::gacha::imports::PullItem;
use crate::gacha::stats_math::average_or_zero;
pub(super) fn standard(signals: &[Row], banners: &BannerCatalog) -> (f64, f64, f64, i32, i32) {
    let catalog = banners;
    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut first_s_rank = true;

    for signal in signals {
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

    (luck_a, luck_s, 0.0, 0, 0)
}
pub(super) fn special(signals: &[Row], banners: &BannerCatalog) -> (f64, f64, f64, i32, i32) {
    let catalog = banners;
    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog.classify(
                    PullPool::Zzz(ZzzGachaType::Special),
                    PullItem::Character(signal.character.unwrap()),
                    signal.timestamp,
                ) == crate::gacha::banner::BannerOutcome::Win;
                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if !is_win {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

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
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    (luck_a, luck_s, win_rate, win_streak, loss_streak)
}
pub(super) fn w_engine(signals: &[Row], banners: &BannerCatalog) -> (f64, f64, f64, i32, i32) {
    let catalog = banners;
    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog.classify(
                    PullPool::Zzz(ZzzGachaType::WEngine),
                    PullItem::WEngine(signal.w_engine.unwrap()),
                    signal.timestamp,
                ) == crate::gacha::banner::BannerOutcome::Win;
                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if !is_win {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

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
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    (luck_a, luck_s, win_rate, win_streak, loss_streak)
}
pub(super) fn bangboo(signals: &[Row], banners: &BannerCatalog) -> (f64, f64, f64, i32, i32) {
    let catalog = banners;
    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    for signal in signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
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

    (luck_a, luck_s, 0.0, 0, 0)
}
pub(super) fn exclusive_rescreening(
    signals: &[Row],
    banners: &BannerCatalog,
) -> (f64, f64, f64, i32, i32) {
    let catalog = banners;
    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog.classify(
                    PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                    PullItem::Character(signal.character.unwrap()),
                    signal.timestamp,
                ) == crate::gacha::banner::BannerOutcome::Win;
                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if !is_win {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

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
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    (luck_a, luck_s, win_rate, win_streak, loss_streak)
}
pub(super) fn w_engine_reverberation(
    signals: &[Row],
    banners: &BannerCatalog,
) -> (f64, f64, f64, i32, i32) {
    let catalog = banners;
    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog.classify(
                    PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                    PullItem::WEngine(signal.w_engine.unwrap()),
                    signal.timestamp,
                ) == crate::gacha::banner::BannerOutcome::Win;
                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if !is_win {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

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
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    (luck_a, luck_s, win_rate, win_streak, loss_streak)
}
#[derive(Clone)]
pub(super) struct Row {
    pub rarity: Option<i32>,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub weapon: Option<i32>,
    pub w_engine: Option<i32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
