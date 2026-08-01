use sqlx::PgConnection;

use crate::{
    database,
    gacha::stats_math::average_or_zero,
    gacha::{
        banner::BannerCatalog,
        imports::{PullItem, PullPool},
    },
    GiGachaType,
};

pub(crate) async fn recalculate_gi_uid(
    uid: i32,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let banners = load_banners(&mut *connection).await?;
    calculate_stats_standard(uid, &mut *connection).await?;
    calculate_stats_character(uid, &banners, &mut *connection).await?;
    calculate_stats_weapon(uid, &banners, &mut *connection).await?;
    calculate_stats_chronicled(uid, &mut *connection).await?;
    Ok(())
}

async fn load_banners(connection: &mut PgConnection) -> anyhow::Result<BannerCatalog> {
    Ok(BannerCatalog::from_gi(
        database::gi::banners::get_all_with_executor(&mut *connection).await?,
    ))
}

async fn calculate_stats_standard(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let wishes = database::gi::wishes::standard::get_infos_by_uid(uid, &mut *connection).await?;

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

    let stat = database::gi::wishes_stats::standard::DbWishesStatStandard {
        uid,
        luck_4,
        luck_5,
    };
    database::gi::wishes_stats::standard::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_character(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let is_win = |item, timestamp| {
        banners
            .classify(
                PullPool::Gi(GiGachaType::Character),
                PullItem::Character(item),
                timestamp,
            )
            .as_win()
    };

    let wishes = database::gi::wishes::character::get_infos_by_uid(uid, &mut *connection).await?;

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

                let Some(is_win) = is_win(wish.character.unwrap(), wish.timestamp) else {
                    continue;
                };
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

    let stat = database::gi::wishes_stats::character::DbWishesStatCharacter {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::gi::wishes_stats::character::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_weapon(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let is_win = |item, timestamp| {
        banners
            .classify(
                PullPool::Gi(GiGachaType::Weapon),
                PullItem::Weapon(item),
                timestamp,
            )
            .as_win()
    };

    let wishes = database::gi::wishes::weapon::get_infos_by_uid(uid, &mut *connection).await?;

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

                let Some(is_win) = is_win(wish.weapon.unwrap(), wish.timestamp) else {
                    continue;
                };
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

    let stat = database::gi::wishes_stats::weapon::DbWishesStatWeapon {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::gi::wishes_stats::weapon::set(&stat, &mut *connection).await?;

    Ok(())
}

async fn calculate_stats_chronicled(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let wishes = database::gi::wishes::chronicled::get_infos_by_uid(uid, &mut *connection).await?;

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

    let stat = database::gi::wishes_stats::chronicled::DbWishesStatChronicled {
        uid,
        luck_4,
        luck_5,
    };
    database::gi::wishes_stats::chronicled::set(&stat, &mut *connection).await?;

    Ok(())
}
