use sqlx::PgConnection;

use crate::database;

pub(crate) async fn recalculate_zzz_uid(
    uid: i32,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    calculate_stats_standard(uid, &mut *connection).await?;
    calculate_stats_special(uid, &mut *connection).await?;
    calculate_stats_w_engine(uid, &mut *connection).await?;
    calculate_stats_bangboo(uid, &mut *connection).await?;
    Ok(())
}

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

async fn calculate_stats_special(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
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

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [1021, 1041, 1101, 1141, 1181, 1211].contains(&signal.character.unwrap()) {
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

async fn calculate_stats_w_engine(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
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

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [14102, 14104, 14110, 14114, 14118, 14121]
                        .contains(&signal.w_engine.unwrap())
                    {
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
