//! Transaction-friendly per-user ZZZ signal-stat calculation for all six pools.
//!
//! Banner-backed pools share one catalog load and the same cross-game fallback:
//! exact featured entries win, permanent items otherwise lose, and
//! non-permanent items default to wins.

use sqlx::PgConnection;

use crate::{
    database,
    gacha::{
        banner::BannerCatalog,
        imports::{PullItem, PullPool},
    },
    ZzzGachaType,
};

/// Recalculates and upserts every ZZZ stat row for one UID.
pub(crate) async fn recalculate_zzz_uid(
    uid: i32,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let catalog = load_banners(&mut *connection).await?;
    calculate_stats_standard(uid, &mut *connection).await?;
    calculate_stats_special(uid, &catalog, &mut *connection).await?;
    calculate_stats_w_engine(uid, &catalog, &mut *connection).await?;
    calculate_stats_bangboo(uid, &mut *connection).await?;
    calculate_stats_exclusive_rescreening(uid, &catalog, &mut *connection).await?;
    calculate_stats_w_engine_reverberation(uid, &catalog, &mut *connection).await?;
    Ok(())
}

/// Loads the pool-aware ZZZ catalog once for the complete UID calculation.
async fn load_banners(connection: &mut PgConnection) -> anyhow::Result<BannerCatalog> {
    Ok(BannerCatalog::from_zzz(
        database::zzz::banners::get_all_with_executor(&mut *connection).await?,
    ))
}

/// Calculates Stable Channel pity averages.
async fn calculate_stats_standard(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let signals = database::zzz::signals::standard::get_infos_by_uid(uid, &mut *connection).await?;

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
    database::zzz::signals_stats::standard::set(&stat, &mut *connection).await?;

    Ok(())
}

/// Calculates Exclusive Channel pity and confirmed banner outcomes.
async fn calculate_stats_special(
    uid: i32,
    catalog: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let signals = database::zzz::signals::special::get_infos_by_uid(uid, &mut *connection).await?;

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
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog
                    .classify(
                        PullPool::Zzz(ZzzGachaType::Special),
                        PullItem::Character(signal.character.unwrap()),
                        signal.timestamp,
                    )
                    .is_win();
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

    let stat = database::zzz::signals_stats::special::DbSignalsStatSpecial {
        uid,
        luck_a,
        luck_s,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::zzz::signals_stats::special::set(&stat, &mut *connection).await?;

    Ok(())
}

/// Calculates W-Engine Channel pity and confirmed banner outcomes.
async fn calculate_stats_w_engine(
    uid: i32,
    catalog: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let signals = database::zzz::signals::w_engine::get_infos_by_uid(uid, &mut *connection).await?;

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
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog
                    .classify(
                        PullPool::Zzz(ZzzGachaType::WEngine),
                        PullItem::WEngine(signal.w_engine.unwrap()),
                        signal.timestamp,
                    )
                    .is_win();
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

    let stat = database::zzz::signals_stats::w_engine::DbSignalsStatWEngine {
        uid,
        luck_a,
        luck_s,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::zzz::signals_stats::w_engine::set(&stat, &mut *connection).await?;

    Ok(())
}

/// Calculates Bangboo Channel pity; this pool has no win/loss metric.
async fn calculate_stats_bangboo(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let signals = database::zzz::signals::bangboo::get_infos_by_uid(uid, &mut *connection).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

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

    let stat = database::zzz::signals_stats::bangboo::DbSignalsStatBangboo {
        uid,
        luck_a,
        luck_s,
    };
    database::zzz::signals_stats::bangboo::set(&stat, &mut *connection).await?;

    Ok(())
}
/// Calculates Exclusive Rescreening pity and confirmed banner outcomes.
async fn calculate_stats_exclusive_rescreening(
    uid: i32,
    catalog: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let signals =
        database::zzz::signals::exclusive_rescreening::get_infos_by_uid(uid, &mut *connection)
            .await?;

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
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog
                    .classify(
                        PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                        PullItem::Character(signal.character.unwrap()),
                        signal.timestamp,
                    )
                    .is_win();
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

    let stat =
        database::zzz::signals_stats::exclusive_rescreening::DbSignalsStatExclusiveRescreening {
            uid,
            luck_a,
            luck_s,
            win_rate,
            win_streak,
            loss_streak,
        };
    database::zzz::signals_stats::exclusive_rescreening::set(&stat, &mut *connection).await?;

    Ok(())
}

/// Calculates W-Engine Reverberation pity and confirmed banner outcomes.
async fn calculate_stats_w_engine_reverberation(
    uid: i32,
    catalog: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let signals =
        database::zzz::signals::w_engine_reverberation::get_infos_by_uid(uid, &mut *connection)
            .await?;

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
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                let is_win = catalog
                    .classify(
                        PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                        PullItem::WEngine(signal.w_engine.unwrap()),
                        signal.timestamp,
                    )
                    .is_win();
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

    let stat =
        database::zzz::signals_stats::w_engine_reverberation::DbSignalsStatWEngineReverberation {
            uid,
            luck_a,
            luck_s,
            win_rate,
            win_streak,
            loss_streak,
        };
    database::zzz::signals_stats::w_engine_reverberation::set(&stat, &mut *connection).await?;

    Ok(())
}
