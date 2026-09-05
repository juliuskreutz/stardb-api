use super::*;
use crate::gacha::imports::PullItem;
#[path = "zzz_baseline.rs"]
mod baseline;
#[test]
fn synthetic_stats_match_pre_refactor_every_pool() {
    let banners = BannerCatalog::default();
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    4
                } else if i % 7 == 0 {
                    3
                } else {
                    2
                }),
                character: Some(if i % 2 == 0 { 1021 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_pity(&rows, |r| r.rarity.unwrap(), 3, 4, true);
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::standard(&rows, &banners),
            "zzz/standard {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    4
                } else if i % 7 == 0 {
                    3
                } else {
                    2
                }),
                character: Some(if i % 2 == 0 { 1021 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_event(
            &rows,
            |r| r.rarity.unwrap(),
            |r| {
                Some(banners.classify(
                    PullPool::Zzz(ZzzGachaType::Special),
                    PullItem::Character(r.character.unwrap()),
                    r.timestamp,
                ))
            },
            3,
            4,
            false,
        );
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::special(&rows, &banners),
            "zzz/special {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    4
                } else if i % 7 == 0 {
                    3
                } else {
                    2
                }),
                character: Some(if i % 2 == 0 { 1021 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_event(
            &rows,
            |r| r.rarity.unwrap(),
            |r| {
                Some(banners.classify(
                    PullPool::Zzz(ZzzGachaType::WEngine),
                    PullItem::WEngine(r.w_engine.unwrap()),
                    r.timestamp,
                ))
            },
            3,
            4,
            false,
        );
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::w_engine(&rows, &banners),
            "zzz/w_engine {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    4
                } else if i % 7 == 0 {
                    3
                } else {
                    2
                }),
                character: Some(if i % 2 == 0 { 1021 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_pity(&rows, |r| r.rarity.unwrap(), 3, 4, false);
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::bangboo(&rows, &banners),
            "zzz/bangboo {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    4
                } else if i % 7 == 0 {
                    3
                } else {
                    2
                }),
                character: Some(if i % 2 == 0 { 1021 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_event(
            &rows,
            |r| r.rarity.unwrap(),
            |r| {
                Some(banners.classify(
                    PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                    PullItem::Character(r.character.unwrap()),
                    r.timestamp,
                ))
            },
            3,
            4,
            false,
        );
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::exclusive_rescreening(&rows, &banners),
            "zzz/exclusive_rescreening {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    4
                } else if i % 7 == 0 {
                    3
                } else {
                    2
                }),
                character: Some(if i % 2 == 0 { 1021 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_event(
            &rows,
            |r| r.rarity.unwrap(),
            |r| {
                Some(banners.classify(
                    PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                    PullItem::WEngine(r.w_engine.unwrap()),
                    r.timestamp,
                ))
            },
            3,
            4,
            false,
        );
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::w_engine_reverberation(&rows, &banners),
            "zzz/w_engine_reverberation {length}"
        );
    }
}
