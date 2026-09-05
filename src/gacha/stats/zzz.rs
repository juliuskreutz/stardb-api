//! Transaction-friendly per-user ZZZ signal-stat calculation for all six pools.
//!
//! Banner-backed pools share one catalog load and the same cross-game fallback:
//! exact featured entries win, permanent items otherwise lose, and
//! non-permanent items default to wins.

use sqlx::PgConnection;

use crate::{
    database,
    gacha::{banner::BannerCatalog, imports::PullPool},
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

    let scan = super::scan::scan_pity(&signals, |row| row.rarity, 3, 4, true);
    let (luck_a, luck_s) = (scan.low, scan.high);
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

    let scan = super::scan::scan_event(
        &signals,
        |row| row.rarity,
        |row| {
            Some(catalog.classify(
                PullPool::Zzz(ZzzGachaType::Special),
                row.item,
                row.timestamp,
            ))
        },
        3,
        4,
        false,
    );
    let (luck_a, luck_s) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

    let scan = super::scan::scan_event(
        &signals,
        |row| row.rarity,
        |row| {
            Some(catalog.classify(
                PullPool::Zzz(ZzzGachaType::WEngine),
                row.item,
                row.timestamp,
            ))
        },
        3,
        4,
        false,
    );
    let (luck_a, luck_s) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

    let scan = super::scan::scan_pity(&signals, |row| row.rarity, 3, 4, false);
    let (luck_a, luck_s) = (scan.low, scan.high);
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

    let scan = super::scan::scan_event(
        &signals,
        |row| row.rarity,
        |row| {
            Some(catalog.classify(
                PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                row.item,
                row.timestamp,
            ))
        },
        3,
        4,
        false,
    );
    let (luck_a, luck_s) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

    let scan = super::scan::scan_event(
        &signals,
        |row| row.rarity,
        |row| {
            Some(catalog.classify(
                PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                row.item,
                row.timestamp,
            ))
        },
        3,
        4,
        false,
    );
    let (luck_a, luck_s) = (scan.low, scan.high);
    let (win_rate, win_streak, loss_streak) = (scan.win_rate, scan.win_streak, scan.loss_streak);
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

#[cfg(test)]
#[path = "zzz_golden.rs"]
mod golden_tests;
