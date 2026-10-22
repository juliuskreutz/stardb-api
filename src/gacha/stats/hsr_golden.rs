use super::*;
use crate::gacha::imports::PullItem;
#[path = "hsr_baseline.rs"]
mod baseline;
#[test]
fn synthetic_stats_match_pre_refactor_every_pool() {
    let banners = BannerCatalog::default();
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    5
                } else if i % 7 == 0 {
                    4
                } else {
                    3
                }),
                character: Some(if i % 2 == 0 { 1209 } else { 9001 }),
                light_cone: Some(if i % 2 == 0 { 23000 } else { 9001 }),
                weapon: Some(if i % 2 == 0 { 15502 } else { 9001 }),
                w_engine: Some(if i % 2 == 0 { 14102 } else { 9001 }),
                timestamp: chrono::DateTime::from_timestamp(1700000000 + i, 0).unwrap(),
            })
            .collect();
        let scan = super::super::scan::scan_pity(&rows, |r| r.rarity.unwrap(), 4, 5, false);
        assert_eq!(
            (
                scan.low,
                scan.high,
                scan.win_rate,
                scan.win_streak,
                scan.loss_streak
            ),
            baseline::standard(&rows, &banners),
            "hsr/standard {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    5
                } else if i % 7 == 0 {
                    4
                } else {
                    3
                }),
                character: Some(if i % 2 == 0 { 1209 } else { 9001 }),
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
                    PullPool::Hsr(GachaType::Special),
                    PullItem::Character(r.character.unwrap()),
                    r.timestamp,
                ))
            },
            4,
            5,
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
            "hsr/special {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    5
                } else if i % 7 == 0 {
                    4
                } else {
                    3
                }),
                character: Some(if i % 2 == 0 { 1209 } else { 9001 }),
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
                    PullPool::Hsr(GachaType::Lc),
                    PullItem::LightCone(r.light_cone.unwrap()),
                    r.timestamp,
                ))
            },
            4,
            5,
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
            baseline::lc(&rows, &banners),
            "hsr/lc {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    5
                } else if i % 7 == 0 {
                    4
                } else {
                    3
                }),
                character: Some(if i % 2 == 0 { 1209 } else { 9001 }),
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
                    PullPool::Hsr(GachaType::Collab),
                    PullItem::Character(r.character.unwrap()),
                    r.timestamp,
                ))
            },
            4,
            5,
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
            baseline::collab(&rows, &banners),
            "hsr/collab {length}"
        );
    }
    for length in [0, 1, 2, 10, 90, 300] {
        let rows: Vec<_> = (0..length)
            .map(|i| baseline::Row {
                rarity: Some(if i % 3 == 0 {
                    5
                } else if i % 7 == 0 {
                    4
                } else {
                    3
                }),
                character: Some(if i % 2 == 0 { 1209 } else { 9001 }),
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
                    PullPool::Hsr(GachaType::CollabLc),
                    PullItem::LightCone(r.light_cone.unwrap()),
                    r.timestamp,
                ))
            },
            4,
            5,
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
            baseline::collab_lc(&rows, &banners),
            "hsr/collab_lc {length}"
        );
    }
}
