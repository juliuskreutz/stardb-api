// Preserve original conversion/borrow syntax in this frozen reference implementation.
#![allow(clippy::useless_conversion, clippy::needless_borrow)]
#![allow(unused_variables)]
//! Frozen tracker JSON reference extracted from 49daa02, before the table rewrite.
use super::*;
use crate::gacha::banner::BannerOutcome;
fn classify_win(
    catalog: &BannerCatalog,
    pool: GachaType,
    item: PullItem,
    timestamp: DateTime<Utc>,
    guarantee: &mut bool,
) -> WinType {
    match catalog.classify(PullPool::Hsr(pool), item, timestamp) {
        BannerOutcome::Win if *guarantee => {
            *guarantee = false;
            WinType::Guarantee
        }
        BannerOutcome::Win => WinType::Win,
        BannerOutcome::Loss if *guarantee => {
            *guarantee = false;
            WinType::Guarantee
        }
        BannerOutcome::Loss => {
            *guarantee = true;
            WinType::Loss
        }
    }
}

pub(super) fn departure(rows: Vec<Warp>, banner_catalog: &BannerCatalog) -> Warps {
    let mut departure = Warps::default();
    let mut departure_pull = 0;
    let mut departure_pull_4 = 0;
    let mut departure_pull_5 = 0;

    for warp in rows {
        let mut warp: Warp = warp.into();

        departure_pull += 1;
        departure_pull_4 += 1;
        departure_pull_5 += 1;

        warp.pull = departure_pull;
        warp.pull_4 = departure_pull_4;
        warp.pull_5 = departure_pull_5;

        match warp.rarity {
            4 => departure_pull_4 = 0,
            5 => departure_pull_5 = 0,
            _ => {}
        }

        departure.warps.push(warp);
    }

    departure.count = departure.warps.len();
    departure
}
pub(super) fn standard(rows: Vec<Warp>, banner_catalog: &BannerCatalog) -> Warps {
    let mut standard = Warps::default();
    let mut standard_pull = 0;
    let mut standard_pull_4 = 0;
    let mut standard_pull_5 = 0;

    for warp in rows {
        let mut warp: Warp = warp.into();

        standard_pull += 1;
        standard_pull_4 += 1;
        standard_pull_5 += 1;

        warp.pull = standard_pull;
        warp.pull_4 = standard_pull_4;
        warp.pull_5 = standard_pull_5;

        match warp.rarity {
            4 => standard_pull_4 = 0,
            5 => standard_pull_5 = 0,
            _ => {}
        }

        standard.warps.push(warp);
    }

    standard.pull_4 = standard_pull_4;
    standard.max_pull_4 = 10;
    standard.probability_4 = if standard_pull_4 < 9 { 5.1 } else { 100.0 };

    standard.pull_5 = standard_pull_5;
    standard.max_pull_5 = 90;
    standard.probability_5 = if standard_pull_5 < 89 {
        0.6 + 6.0 * standard_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    standard.count = standard.warps.len();
    standard
}
pub(super) fn special(rows: Vec<Warp>, banner_catalog: &BannerCatalog) -> Warps {
    let mut special = Warps::default();
    let mut special_pull = 0;
    let mut special_pull_4 = 0;
    let mut special_pull_5 = 0;
    let mut guarantee = false;

    for warp in rows {
        let mut warp: Warp = warp.into();

        special_pull += 1;
        special_pull_4 += 1;
        special_pull_5 += 1;

        warp.pull = special_pull;
        warp.pull_4 = special_pull_4;
        warp.pull_5 = special_pull_5;

        match warp.rarity {
            4 => special_pull_4 = 0,
            5 => {
                special_pull_5 = 0;

                warp.win = Some(classify_win(
                    &banner_catalog,
                    GachaType::Special,
                    PullItem::Character(warp.item_id),
                    warp.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        special.warps.push(warp);
    }

    special.pull_4 = special_pull_4;
    special.max_pull_4 = 10;
    special.probability_4 = if special_pull_4 < 9 { 5.1 } else { 100.0 };

    special.pull_5 = special_pull_5;
    special.max_pull_5 = 90;
    special.probability_5 = if special_pull_5 < 89 {
        0.6 + 6.0 * special_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    special.count = special.warps.len();
    special
}
pub(super) fn lc(rows: Vec<Warp>, banner_catalog: &BannerCatalog) -> Warps {
    let mut lc = Warps::default();
    let mut lc_pull = 0;
    let mut lc_pull_4 = 0;
    let mut lc_pull_5 = 0;
    let mut guarantee = false;

    for warp in rows {
        let mut warp: Warp = warp.into();

        lc_pull += 1;
        lc_pull_4 += 1;
        lc_pull_5 += 1;

        warp.pull = lc_pull;
        warp.pull_4 = lc_pull_4;
        warp.pull_5 = lc_pull_5;

        match warp.rarity {
            4 => lc_pull_4 = 0,
            5 => {
                lc_pull_5 = 0;

                warp.win = Some(classify_win(
                    &banner_catalog,
                    GachaType::Lc,
                    PullItem::LightCone(warp.item_id),
                    warp.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        lc.warps.push(warp);
    }

    lc.pull_4 = lc_pull_4;
    lc.max_pull_4 = 10;
    lc.probability_4 = if lc_pull_4 < 9 { 6.6 } else { 100.0 };

    lc.pull_5 = lc_pull_5;
    lc.max_pull_5 = 80;
    lc.probability_5 = if lc_pull_5 < 79 {
        0.8 + 7.0 * lc_pull_5.saturating_sub(64) as f64
    } else {
        100.0
    };

    lc.count = lc.warps.len();
    lc
}
pub(super) fn collab(rows: Vec<Warp>, banner_catalog: &BannerCatalog) -> Warps {
    let mut collab = Warps::default();
    let mut collab_pull = 0;
    let mut collab_pull_4 = 0;
    let mut collab_pull_5 = 0;
    let mut collab_guarantee = false;

    for warp in rows {
        let mut warp: Warp = warp.into();

        collab_pull += 1;
        collab_pull_4 += 1;
        collab_pull_5 += 1;

        warp.pull = collab_pull;
        warp.pull_4 = collab_pull_4;
        warp.pull_5 = collab_pull_5;

        match warp.rarity {
            4 => collab_pull_4 = 0,
            5 => {
                collab_pull_5 = 0;

                warp.win = Some(classify_win(
                    &banner_catalog,
                    GachaType::Collab,
                    PullItem::Character(warp.item_id),
                    warp.timestamp,
                    &mut collab_guarantee,
                ));
            }
            _ => {}
        }

        collab.warps.push(warp);
    }

    collab.pull_4 = collab_pull_4;
    collab.max_pull_4 = 10;
    collab.probability_4 = if collab_pull_4 < 9 { 5.1 } else { 100.0 };

    collab.pull_5 = collab_pull_5;
    collab.max_pull_5 = 90;
    collab.probability_5 = if collab_pull_5 < 89 {
        0.6 + 6.0 * collab_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    collab.count = collab.warps.len();
    collab
}
pub(super) fn collab_lc(rows: Vec<Warp>, banner_catalog: &BannerCatalog) -> Warps {
    let mut collab_lc = Warps::default();
    let mut collab_lc_pull = 0;
    let mut collab_lc_pull_4 = 0;
    let mut collab_lc_pull_5 = 0;
    let mut collab_lc_guarantee = false;

    for warp in rows {
        let mut warp: Warp = warp.into();

        collab_lc_pull += 1;
        collab_lc_pull_4 += 1;
        collab_lc_pull_5 += 1;

        warp.pull = collab_lc_pull;
        warp.pull_4 = collab_lc_pull_4;
        warp.pull_5 = collab_lc_pull_5;

        match warp.rarity {
            4 => collab_lc_pull_4 = 0,
            5 => {
                collab_lc_pull_5 = 0;

                warp.win = Some(classify_win(
                    &banner_catalog,
                    GachaType::CollabLc,
                    PullItem::LightCone(warp.item_id),
                    warp.timestamp,
                    &mut collab_lc_guarantee,
                ));
            }
            _ => {}
        }

        collab_lc.warps.push(warp);
    }

    collab_lc.pull_4 = collab_lc_pull_4;
    collab_lc.max_pull_4 = 10;
    collab_lc.probability_4 = if collab_lc_pull_4 < 9 { 6.6 } else { 100.0 };

    collab_lc.pull_5 = collab_lc_pull_5;
    collab_lc.max_pull_5 = 80;
    collab_lc.probability_5 = if collab_lc_pull_5 < 79 {
        0.8 + 7.0 * collab_lc_pull_5.saturating_sub(64) as f64
    } else {
        100.0
    };

    collab_lc.count = collab_lc.warps.len();
    collab_lc
}
