// Preserve original conversion/borrow syntax in this frozen reference implementation.
#![allow(clippy::useless_conversion, clippy::needless_borrow)]
#![allow(unused_variables)]
//! Frozen tracker JSON reference extracted from 49daa02, before the table rewrite.
use super::*;
use crate::gacha::banner::BannerOutcome;
fn classify_win(
    catalog: &BannerCatalog,
    pool: ZzzGachaType,
    item: PullItem,
    timestamp: DateTime<Utc>,
    guarantee: &mut bool,
) -> WinType {
    match catalog.classify(PullPool::Zzz(pool), item, timestamp) {
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

pub(super) fn standard(rows: Vec<Signal>, banner_catalog: &BannerCatalog) -> Signals {
    let mut standard = Signals::default();
    let mut standard_pull = 0;
    let mut standard_pull_a = 0;
    let mut standard_pull_s = 0;

    for signal in rows {
        let mut signal: Signal = signal.into();

        standard_pull += 1;
        standard_pull_a += 1;
        standard_pull_s += 1;

        signal.pull = standard_pull;
        signal.pull_4 = standard_pull_a;
        signal.pull_5 = standard_pull_s;

        match signal.rarity {
            3 => standard_pull_a = 0,
            4 => {
                standard_pull_a = 0;
                standard_pull_s = 0;
            }
            _ => {}
        }

        standard.signals.push(signal);
    }

    standard.pull_4 = standard_pull_a;
    standard.max_pull_4 = 10;
    standard.probability_4 = if standard_pull_a < 9 { 9.4 } else { 100.0 };

    standard.pull_5 = standard_pull_s;
    standard.max_pull_5 = 90;
    standard.probability_5 = if standard_pull_s < 89 {
        0.6 + 6.0 * standard_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    standard.count = standard.signals.len();
    standard
}
pub(super) fn special(rows: Vec<Signal>, banner_catalog: &BannerCatalog) -> Signals {
    let mut special = Signals::default();
    let mut special_pull = 0;
    let mut special_pull_a = 0;
    let mut special_pull_s = 0;
    let mut guarantee = false;

    for signal in rows {
        let mut signal: Signal = signal.into();

        special_pull += 1;
        special_pull_a += 1;
        special_pull_s += 1;

        signal.pull = special_pull;
        signal.pull_4 = special_pull_a;
        signal.pull_5 = special_pull_s;

        match signal.rarity {
            3 => special_pull_a = 0,
            4 => {
                special_pull_a = 0;
                special_pull_s = 0;

                signal.win = Some(classify_win(
                    &banner_catalog,
                    ZzzGachaType::Special,
                    PullItem::Character(signal.item_id),
                    signal.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        special.signals.push(signal);
    }

    special.pull_4 = special_pull_a;
    special.max_pull_4 = 10;
    special.probability_4 = if special_pull_a < 9 { 9.4 } else { 100.0 };

    special.pull_5 = special_pull_s;
    special.max_pull_5 = 90;
    special.probability_5 = if special_pull_s < 89 {
        0.6 + 6.0 * special_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    special.count = special.signals.len();
    special
}
pub(super) fn w_engine(rows: Vec<Signal>, banner_catalog: &BannerCatalog) -> Signals {
    let mut w_engine = Signals::default();
    let mut w_engine_pull = 0;
    let mut w_engine_pull_a = 0;
    let mut w_engine_pull_s = 0;
    let mut guarantee = false;

    for signal in rows {
        let mut signal: Signal = signal.into();

        w_engine_pull += 1;
        w_engine_pull_a += 1;
        w_engine_pull_s += 1;

        signal.pull = w_engine_pull;
        signal.pull_4 = w_engine_pull_a;
        signal.pull_5 = w_engine_pull_s;

        match signal.rarity {
            3 => w_engine_pull_a = 0,
            4 => {
                w_engine_pull_a = 0;
                w_engine_pull_s = 0;

                signal.win = Some(classify_win(
                    &banner_catalog,
                    ZzzGachaType::WEngine,
                    PullItem::WEngine(signal.item_id),
                    signal.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        w_engine.signals.push(signal);
    }

    w_engine.pull_4 = w_engine_pull_a;
    w_engine.max_pull_4 = 10;
    w_engine.probability_4 = if w_engine_pull_a < 9 { 15.0 } else { 100.0 };

    w_engine.pull_5 = w_engine_pull_s;
    w_engine.max_pull_5 = 80;
    w_engine.probability_5 = if w_engine_pull_s < 79 {
        1.0 + 7.0 * w_engine_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    w_engine.count = w_engine.signals.len();
    w_engine
}
pub(super) fn bangboo(rows: Vec<Signal>, banner_catalog: &BannerCatalog) -> Signals {
    let mut bangboo = Signals::default();
    let mut bangboo_pull = 0;
    let mut bangboo_pull_a = 0;
    let mut bangboo_pull_s = 0;

    for signal in rows {
        let mut signal: Signal = signal.into();

        bangboo_pull += 1;
        bangboo_pull_a += 1;
        bangboo_pull_s += 1;

        signal.pull = bangboo_pull;
        signal.pull_4 = bangboo_pull_a;
        signal.pull_5 = bangboo_pull_s;

        match signal.rarity {
            3 => bangboo_pull_a = 0,
            4 => {
                bangboo_pull_a = 0;
                bangboo_pull_s = 0;
            }
            _ => {}
        }

        bangboo.signals.push(signal);
    }

    bangboo.pull_4 = bangboo_pull_a;
    bangboo.max_pull_4 = 10;
    bangboo.probability_4 = if bangboo_pull_a < 9 { 15.0 } else { 100.0 };

    bangboo.pull_5 = bangboo_pull_s;
    bangboo.max_pull_5 = 80;
    bangboo.probability_5 = if bangboo_pull_s < 79 {
        1.0 + 7.0 * bangboo_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    bangboo.count = bangboo.signals.len();
    bangboo
}
pub(super) fn exclusive_rescreening(rows: Vec<Signal>, banner_catalog: &BannerCatalog) -> Signals {
    let mut exclusive_rescreening = Signals::default();
    let mut exclusive_rescreening_pull = 0;
    let mut exclusive_rescreening_pull_a = 0;
    let mut exclusive_rescreening_pull_s = 0;
    let mut guarantee = false;

    for signal in rows {
        let mut signal: Signal = signal.into();

        exclusive_rescreening_pull += 1;
        exclusive_rescreening_pull_a += 1;
        exclusive_rescreening_pull_s += 1;

        signal.pull = exclusive_rescreening_pull;
        signal.pull_4 = exclusive_rescreening_pull_a;
        signal.pull_5 = exclusive_rescreening_pull_s;

        match signal.rarity {
            3 => exclusive_rescreening_pull_a = 0,
            4 => {
                exclusive_rescreening_pull_a = 0;
                exclusive_rescreening_pull_s = 0;

                signal.win = Some(classify_win(
                    &banner_catalog,
                    ZzzGachaType::ExclusiveRescreening,
                    PullItem::Character(signal.item_id),
                    signal.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        exclusive_rescreening.signals.push(signal);
    }

    exclusive_rescreening.pull_4 = exclusive_rescreening_pull_a;
    exclusive_rescreening.max_pull_4 = 10;
    exclusive_rescreening.probability_4 = if exclusive_rescreening_pull_a < 9 {
        9.4
    } else {
        100.0
    };

    exclusive_rescreening.pull_5 = exclusive_rescreening_pull_s;
    exclusive_rescreening.max_pull_5 = 90;
    exclusive_rescreening.probability_5 = if exclusive_rescreening_pull_s < 89 {
        0.6 + 6.0 * exclusive_rescreening_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    exclusive_rescreening.count = exclusive_rescreening.signals.len();
    exclusive_rescreening
}
pub(super) fn w_engine_reverberation(rows: Vec<Signal>, banner_catalog: &BannerCatalog) -> Signals {
    let mut w_engine_reverberation = Signals::default();
    let mut w_engine_reverberation_pull = 0;
    let mut w_engine_reverberation_pull_a = 0;
    let mut w_engine_reverberation_pull_s = 0;
    let mut guarantee = false;

    for signal in rows {
        let mut signal: Signal = signal.into();

        w_engine_reverberation_pull += 1;
        w_engine_reverberation_pull_a += 1;
        w_engine_reverberation_pull_s += 1;

        signal.pull = w_engine_reverberation_pull;
        signal.pull_4 = w_engine_reverberation_pull_a;
        signal.pull_5 = w_engine_reverberation_pull_s;

        match signal.rarity {
            3 => w_engine_reverberation_pull_a = 0,
            4 => {
                w_engine_reverberation_pull_a = 0;
                w_engine_reverberation_pull_s = 0;

                signal.win = Some(classify_win(
                    &banner_catalog,
                    ZzzGachaType::WEngineReverberation,
                    PullItem::WEngine(signal.item_id),
                    signal.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        w_engine_reverberation.signals.push(signal);
    }

    w_engine_reverberation.pull_4 = w_engine_reverberation_pull_a;
    w_engine_reverberation.max_pull_4 = 10;
    w_engine_reverberation.probability_4 = if w_engine_reverberation_pull_a < 9 {
        15.0
    } else {
        100.0
    };

    w_engine_reverberation.pull_5 = w_engine_reverberation_pull_s;
    w_engine_reverberation.max_pull_5 = 80;
    w_engine_reverberation.probability_5 = if w_engine_reverberation_pull_s < 79 {
        1.0 + 7.0 * w_engine_reverberation_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    w_engine_reverberation.count = w_engine_reverberation.signals.len();
    w_engine_reverberation
}
