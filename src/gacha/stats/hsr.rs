//! Transaction-friendly per-user HSR gacha-stat calculation.
//!
//! Pity luck includes every high-rarity pull. Win-rate and streak calculations
//! use the shared cross-game banner rule: exact featured entries win, permanent
//! items otherwise lose, and non-permanent items default to wins.

use sqlx::PgConnection;

use crate::{
    database,
    gacha::{banner::BannerCatalog, imports::PullPool},
    GachaType,
};

/// Recalculates and upserts every HSR stat row for one UID.
/// Uses one banner snapshot and the caller’s connection without beginning or
/// committing a transaction. A read/decode/upsert failure propagates immediately;
/// import callers wrap all affected pools in their transaction for atomicity.
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

/// Loads the pool-aware HSR catalog once for the complete UID calculation.
async fn load_banners(connection: &mut PgConnection) -> anyhow::Result<BannerCatalog> {
    Ok(BannerCatalog::from_hsr(
        database::banners::get_all_with_executor(&mut *connection).await?,
    ))
}

/// Calculates permanent-pool pity averages.
/// Reads pull-ID-ordered history and upserts this pool’s local stats on the caller’s connection.
async fn calculate_stats_standard(uid: i32, connection: &mut PgConnection) -> anyhow::Result<()> {
    let warps = database::warps::standard::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_pity(&warps, |row| row.rarity, 4, 5, false);
    let (luck_4, luck_5) = (scan.low, scan.high);
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

/// Calculates character-event pity, win rate, and confirmed streaks.
/// Reads pull-ID-ordered history and upserts this pool’s local stats on the caller’s connection.
async fn calculate_stats_special(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let warps = database::warps::special::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_event(
        &warps,
        |row| row.rarity,
        |row| Some(banners.classify(PullPool::Hsr(GachaType::Special), row.item, row.timestamp)),
        4,
        5,
        false,
    );
    let (luck_4, luck_5) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

/// Calculates Light Cone-event pity, win rate, and confirmed streaks.
/// Reads pull-ID-ordered history and upserts this pool’s local stats on the caller’s connection.
async fn calculate_stats_lc(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let warps = database::warps::lc::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_event(
        &warps,
        |row| row.rarity,
        |row| Some(banners.classify(PullPool::Hsr(GachaType::Lc), row.item, row.timestamp)),
        4,
        5,
        false,
    );
    let (luck_4, luck_5) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

/// Calculates collab character-event stats independently from normal banners.
/// Reads pull-ID-ordered history and upserts this pool’s local stats on the caller’s connection.
async fn calculate_stats_collab(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let warps = database::warps::collab::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_event(
        &warps,
        |row| row.rarity,
        |row| Some(banners.classify(PullPool::Hsr(GachaType::Collab), row.item, row.timestamp)),
        4,
        5,
        false,
    );
    let (luck_4, luck_5) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

/// Calculates collab Light Cone stats independently from normal banners.
/// Reads pull-ID-ordered history and upserts this pool’s local stats on the caller’s connection.
async fn calculate_stats_collab_lc(
    uid: i32,
    banners: &BannerCatalog,
    connection: &mut PgConnection,
) -> anyhow::Result<()> {
    let warps = database::warps::collab_lc::get_infos_by_uid(uid, &mut *connection).await?;

    let scan = super::scan::scan_event(
        &warps,
        |row| row.rarity,
        |row| Some(banners.classify(PullPool::Hsr(GachaType::CollabLc), row.item, row.timestamp)),
        4,
        5,
        false,
    );
    let (luck_4, luck_5) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

#[cfg(test)]
#[path = "hsr_golden.rs"]
mod golden_tests;
