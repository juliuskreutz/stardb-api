use super::*;
#[test]
fn all_pool_json_matches_pre_refactor() {
    let catalog = BannerCatalog::default();
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Signal {
                    r#type: SignalType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        4
                    } else if i % 7 == 0 {
                        3
                    } else {
                        2
                    },
                    item_id: if i % 2 == 0 { 1021 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), ZzzGachaType::Standard, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::standard(rows(), &catalog)).unwrap(),
            "standard, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Signal {
                    r#type: SignalType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        4
                    } else if i % 7 == 0 {
                        3
                    } else {
                        2
                    },
                    item_id: if i % 2 == 0 { 1021 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), ZzzGachaType::Special, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::special(rows(), &catalog)).unwrap(),
            "special, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Signal {
                    r#type: SignalType::WEngine,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        4
                    } else if i % 7 == 0 {
                        3
                    } else {
                        2
                    },
                    item_id: if i % 2 == 0 { 14102 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), ZzzGachaType::WEngine, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::w_engine(rows(), &catalog)).unwrap(),
            "w_engine, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Signal {
                    r#type: SignalType::Bangboo,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        4
                    } else if i % 7 == 0 {
                        3
                    } else {
                        2
                    },
                    item_id: if i % 2 == 0 { 1021 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), ZzzGachaType::Bangboo, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::bangboo(rows(), &catalog)).unwrap(),
            "bangboo, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Signal {
                    r#type: SignalType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        4
                    } else if i % 7 == 0 {
                        3
                    } else {
                        2
                    },
                    item_id: if i % 2 == 0 { 1021 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(
                rows(),
                ZzzGachaType::ExclusiveRescreening,
                &catalog
            ))
            .unwrap(),
            serde_json::to_vec(&baseline::exclusive_rescreening(rows(), &catalog)).unwrap(),
            "exclusive_rescreening, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Signal {
                    r#type: SignalType::WEngine,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        4
                    } else if i % 7 == 0 {
                        3
                    } else {
                        2
                    },
                    item_id: if i % 2 == 0 { 14102 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(
                rows(),
                ZzzGachaType::WEngineReverberation,
                &catalog
            ))
            .unwrap(),
            serde_json::to_vec(&baseline::w_engine_reverberation(rows(), &catalog)).unwrap(),
            "w_engine_reverberation, {length}"
        );
    }
}
#[test]
fn soft_and_hard_pity_boundaries() {
    let p = Pity {
        base_4: 5.1,
        base_5: 0.6,
        gain_5: 6.0,
        soft_start_5: 72,
        hard_4: 9,
        hard_5: 89,
        max_4: 10,
        max_5: 90,
    };
    assert_eq!(p.probabilities(0, 73).1, 6.6);
    assert_eq!(p.probabilities(9, 89), (100.0, 100.0));
}
