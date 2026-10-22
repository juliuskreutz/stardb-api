use super::*;
type ReferenceBuilder = fn(Vec<Wish>, &BannerCatalog) -> Wishes;
#[test]
fn all_pool_json_matches_pre_refactor() {
    let catalog = BannerCatalog::default();
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Wish {
                    r#type: WishType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        5
                    } else if i % 7 == 0 {
                        4
                    } else {
                        3
                    },
                    item_id: if i % 2 == 0 { 10000042 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), GiGachaType::Beginner, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::beginner(rows(), &catalog)).unwrap(),
            "beginner, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Wish {
                    r#type: WishType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        5
                    } else if i % 7 == 0 {
                        4
                    } else {
                        3
                    },
                    item_id: if i % 2 == 0 { 10000042 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), GiGachaType::Standard, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::standard(rows(), &catalog)).unwrap(),
            "standard, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Wish {
                    r#type: WishType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        5
                    } else if i % 7 == 0 {
                        4
                    } else {
                        3
                    },
                    item_id: if i % 2 == 0 { 10000042 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), GiGachaType::Character, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::character(rows(), &catalog)).unwrap(),
            "character, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Wish {
                    r#type: WishType::Weapon,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        5
                    } else if i % 7 == 0 {
                        4
                    } else {
                        3
                    },
                    item_id: if i % 2 == 0 { 15502 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), GiGachaType::Weapon, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::weapon(rows(), &catalog)).unwrap(),
            "weapon, {length}"
        );
    }
    for length in [0, 1, 10, 65, 73, 79, 89, 95] {
        let rows = || {
            (0..length)
                .map(|i| Wish {
                    r#type: WishType::Character,
                    id: i.to_string(),
                    name: "fixture".into(),
                    rarity: if i % 11 == 0 {
                        5
                    } else if i % 7 == 0 {
                        4
                    } else {
                        3
                    },
                    item_id: if i % 2 == 0 { 10000042 } else { 9001 },
                    timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                    pull: 0,
                    pull_4: 0,
                    pull_5: 0,
                    win: None,
                })
                .collect()
        };
        assert_eq!(
            serde_json::to_vec(&build_set(rows(), GiGachaType::Chronicled, &catalog)).unwrap(),
            serde_json::to_vec(&baseline::chronicled(rows(), &catalog)).unwrap(),
            "chronicled, {length}"
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

#[test]
fn drought_histories_preserve_each_pools_soft_and_hard_pity() {
    let catalog = BannerCatalog::default();
    // No pity resets: these lengths actually reach both weapon and character
    // thresholds, unlike the mixed histories used by the annotation fixture.
    let cases: [(GiGachaType, ReferenceBuilder); 5] = [
        (GiGachaType::Beginner, baseline::beginner),
        (GiGachaType::Standard, baseline::standard),
        (GiGachaType::Character, baseline::character),
        (GiGachaType::Weapon, baseline::weapon),
        (GiGachaType::Chronicled, baseline::chronicled),
    ];
    for (kind, reference) in cases {
        for length in [65, 73, 79, 89, 95] {
            let rows = || {
                (0..length)
                    .map(|i| Wish {
                        r#type: WishType::Character,
                        id: i.to_string(),
                        name: "drought".into(),
                        rarity: 3,
                        item_id: 9001,
                        timestamp: DateTime::from_timestamp(1700000000 + i as i64, 0).unwrap(),
                        pull: 0,
                        pull_4: 0,
                        pull_5: 0,
                        win: None,
                    })
                    .collect()
            };
            assert_eq!(
                serde_json::to_vec(&build_set(rows(), kind, &catalog)).unwrap(),
                serde_json::to_vec(&reference(rows(), &catalog)).unwrap(),
                "{kind:?}, drought {length}"
            );
        }
    }
}
