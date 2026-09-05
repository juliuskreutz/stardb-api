//! Transaction-friendly per-user Genshin wish-stat calculation.
//!
//! Limited-pool win metrics use the shared cross-game banner rule: exact
//! featured entries win, permanent items otherwise lose, and non-permanent
//! items default to wins.

use sqlx::PgConnection;

use crate::{
    database,
    gacha::{banner::BannerCatalog, imports::PullPool},
    GiGachaType,
};

/// Recalculates and upserts every Genshin stat row for one UID.
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

/// Loads the pool-aware Genshin catalog once for the complete UID calculation.
async fn load_banners(connection: &mut PgConnection) -> anyhow::Result<BannerCatalog> {
    Ok(BannerCatalog::from_gi(
        database::gi::banners::get_all_with_executor(&mut *connection).await?,
    ))
}

/// Calculates permanent-pool pity averages.
async fn calculate_stats_standard(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let wishes = database::gi::wishes::standard::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_pity(&wishes, |row| row.rarity, 4, 5, false);
    let (luck_4, luck_5) = (scan.low, scan.high);
    let stat = database::gi::wishes_stats::standard::DbWishesStatStandard {
        uid,
        luck_4,
        luck_5,
    };
    database::gi::wishes_stats::standard::set(&stat, &mut *connection).await?;

    Ok(())
}

/// Calculates character-event pity, win rate, and confirmed streaks.
async fn calculate_stats_character(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let wishes = database::gi::wishes::character::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_event(
        &wishes,
        |row| row.rarity,
        |row| {
            Some(banners.classify(
                PullPool::Gi(GiGachaType::Character),
                row.item,
                row.timestamp,
            ))
        },
        4,
        5,
        false,
    );
    let (luck_4, luck_5) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

/// Calculates weapon-event pity, win rate, and confirmed streaks.
async fn calculate_stats_weapon(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let wishes = database::gi::wishes::weapon::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_event(
        &wishes,
        |row| row.rarity,
        |row| Some(banners.classify(PullPool::Gi(GiGachaType::Weapon), row.item, row.timestamp)),
        4,
        5,
        false,
    );
    let (luck_4, luck_5) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

/// Calculates Chronicled Wish pity without applying a different win model.
async fn calculate_stats_chronicled(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let wishes = database::gi::wishes::chronicled::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_pity(&wishes, |row| row.rarity, 4, 5, false);
    let (luck_4, luck_5) = (scan.low, scan.high);
    let stat = database::gi::wishes_stats::chronicled::DbWishesStatChronicled {
        uid,
        luck_4,
        luck_5,
    };
    database::gi::wishes_stats::chronicled::set(&stat, &mut *connection).await?;

    Ok(())
}

#[cfg(test)]
#[path = "gi_golden.rs"]
mod golden_tests;
