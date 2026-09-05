//! Pool-aware banner validation and win/loss classification.
//!
//! Database rows remain game-specific, but every caller classifies through the
//! same normalized catalog so one game's banner cannot affect another pool.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{database, GachaType, GiGachaType, ZzzGachaType};

use super::imports::{PullItem, PullPool};

/// Validates the range and item/pool pairs shared by all banner admin routes.
///
/// Each tuple contains `item_is_present`, its optional pool ID, and the pool IDs
/// permitted for that item kind. At least one featured item must be present.
pub(crate) fn validate_banner(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    item_pools: &[(bool, Option<i32>, &[i32])],
) -> bool {
    start < end
        && item_pools.iter().any(|(present, _, _)| *present)
        && item_pools.iter().all(|(present, pool, allowed)| {
            *present == pool.is_some() && pool.is_none_or(|pool| allowed.contains(&pool))
        })
}

/// Win/loss classification of a high-rarity pull.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BannerOutcome {
    /// The item is actively featured or is not part of the permanent pool.
    Win,
    /// The item is permanent and is not actively featured in this exact pool.
    Loss,
}

/// Shared state, with callers deciding how guaranteed wins affect their output.
#[derive(Default, Debug)]
pub(crate) struct GuaranteeState {
    armed: bool,
}
/// A classified decision after consuming the pool’s guarantee state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuaranteedOutcome {
    Win,
    GuaranteedWin,
    Loss,
}
impl GuaranteeState {
    /// Consumes one high-rarity result and advances the guarantee state.
    /// An armed guarantee overrides either catalog result and is consumed once;
    /// only a non-guaranteed loss arms the next pull.
    pub(crate) fn advance(&mut self, outcome: BannerOutcome) -> GuaranteedOutcome {
        if self.armed {
            self.armed = false;
            return GuaranteedOutcome::GuaranteedWin;
        }
        match outcome {
            BannerOutcome::Win => GuaranteedOutcome::Win,
            BannerOutcome::Loss => {
                self.armed = true;
                GuaranteedOutcome::Loss
            }
        }
    }
}

/// One normalized featured item and its half-open availability range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BannerEntry {
    pub pool: PullPool,
    pub item: PullItem,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Featured items grouped by their exact game and pull pool.
#[derive(Default)]
pub(crate) struct BannerCatalog {
    entries_by_pool: HashMap<PullPool, Vec<BannerEntry>>,
}

impl BannerCatalog {
    /// Builds a catalog from already normalized banner entries.
    pub(crate) fn new(entries: impl IntoIterator<Item = BannerEntry>) -> Self {
        let mut entries_by_pool: HashMap<_, Vec<_>> = HashMap::new();
        for entry in entries {
            entries_by_pool.entry(entry.pool).or_default().push(entry);
        }
        Self { entries_by_pool }
    }

    /// Adapts HSR persistence rows into Special, Light Cone, and collab entries.
    /// Unknown stored pool IDs are logged and skipped per item, never relabeled.
    pub(crate) fn from_hsr(banners: impl IntoIterator<Item = database::banners::DbBanner>) -> Self {
        Self::new(banners.into_iter().flat_map(|banner| {
            let mut entries = Vec::with_capacity(2);
            if let (Some(item), Some(pool)) = (banner.character, banner.character_gacha_type) {
                let pool = match pool {
                    21 => Some(GachaType::Collab),
                    11 => Some(GachaType::Special),
                    unknown => {
                        warn!("Skipping unknown banner pool {unknown}");
                        None
                    }
                };
                if let Some(pool) = pool {
                    entries.push(BannerEntry {
                        pool: PullPool::Hsr(pool),
                        item: PullItem::Character(item),
                        start: banner.start,
                        end: banner.end,
                    });
                }
            }
            if let (Some(item), Some(pool)) = (banner.light_cone, banner.light_cone_gacha_type) {
                let pool = match pool {
                    22 => Some(GachaType::CollabLc),
                    12 => Some(GachaType::Lc),
                    unknown => {
                        warn!("Skipping unknown banner pool {unknown}");
                        None
                    }
                };
                if let Some(pool) = pool {
                    entries.push(BannerEntry {
                        pool: PullPool::Hsr(pool),
                        item: PullItem::LightCone(item),
                        start: banner.start,
                        end: banner.end,
                    });
                }
            }
            entries
        }))
    }

    /// Adapts Genshin persistence rows into Character, Weapon, and Chronicled entries.
    /// Uses the persisted banner vocabulary; unknown IDs are logged and skipped.
    pub(crate) fn from_gi(
        banners: impl IntoIterator<Item = database::gi::banners::DbBanner>,
    ) -> Self {
        Self::new(banners.into_iter().flat_map(|banner| {
            let mut entries = Vec::with_capacity(2);
            if let (Some(item), Some(pool)) = (banner.character, banner.character_gacha_type) {
                let pool = match pool {
                    500 => Some(GiGachaType::Chronicled),
                    301 => Some(GiGachaType::Character),
                    unknown => {
                        warn!("Skipping unknown banner pool {unknown}");
                        None
                    }
                };
                if let Some(pool) = pool {
                    entries.push(BannerEntry {
                        pool: PullPool::Gi(pool),
                        item: PullItem::Character(item),
                        start: banner.start,
                        end: banner.end,
                    });
                }
            }
            if let (Some(item), Some(pool)) = (banner.weapon, banner.weapon_gacha_type) {
                let pool = match pool {
                    500 => Some(GiGachaType::Chronicled),
                    302 => Some(GiGachaType::Weapon),
                    unknown => {
                        warn!("Skipping unknown banner pool {unknown}");
                        None
                    }
                };
                if let Some(pool) = pool {
                    entries.push(BannerEntry {
                        pool: PullPool::Gi(pool),
                        item: PullItem::Weapon(item),
                        start: banner.start,
                        end: banner.end,
                    });
                }
            }
            entries
        }))
    }

    /// Adapts ZZZ persistence rows into its five banner-backed pool catalogs.
    /// Unknown item/pool mappings are logged and skipped without affecting valid siblings.
    pub(crate) fn from_zzz(
        banners: impl IntoIterator<Item = database::zzz::banners::DbBanner>,
    ) -> Self {
        Self::new(banners.into_iter().flat_map(|banner| {
            let mut entries = Vec::with_capacity(3);
            if let (Some(item), Some(pool)) = (banner.character, banner.character_gacha_type) {
                let pool = match pool {
                    102 => Some(ZzzGachaType::ExclusiveRescreening),
                    2 => Some(ZzzGachaType::Special),
                    unknown => {
                        warn!("Skipping unknown banner pool {unknown}");
                        None
                    }
                };
                if let Some(pool) = pool {
                    entries.push(BannerEntry {
                        pool: PullPool::Zzz(pool),
                        item: PullItem::Character(item),
                        start: banner.start,
                        end: banner.end,
                    });
                }
            }
            if let (Some(item), Some(pool)) = (banner.w_engine, banner.w_engine_gacha_type) {
                let pool = match pool {
                    103 => Some(ZzzGachaType::WEngineReverberation),
                    3 => Some(ZzzGachaType::WEngine),
                    unknown => {
                        warn!("Skipping unknown banner pool {unknown}");
                        None
                    }
                };
                if let Some(pool) = pool {
                    entries.push(BannerEntry {
                        pool: PullPool::Zzz(pool),
                        item: PullItem::WEngine(item),
                        start: banner.start,
                        end: banner.end,
                    });
                }
            }
            if let (Some(item), Some(pool)) = (banner.bangboo, banner.bangboo_gacha_type) {
                if pool != 5 {
                    warn!("Skipping unknown bangboo banner pool {pool}");
                    return entries;
                }
                entries.push(BannerEntry {
                    pool: PullPool::Zzz(ZzzGachaType::Bangboo),
                    item: PullItem::Bangboo(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            entries
        }))
    }

    /// Classifies one pull using the same fallback rules for every game.
    ///
    /// Exact featured ranges are `[start, end)` and take precedence over the
    /// permanent-item catalog. This matters for pools such as Chronicled Wish,
    /// where a normally permanent character or weapon can itself be featured.
    /// Without an exact featured match, permanent items are losses and every
    /// other item defaults to a win so incomplete banner history retains the
    /// original tracker behavior.
    pub(crate) fn classify(
        &self,
        pool: PullPool,
        item: PullItem,
        timestamp: DateTime<Utc>,
    ) -> BannerOutcome {
        let is_featured = self.entries_by_pool.get(&pool).is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry.item == item && entry.start <= timestamp && timestamp < entry.end
            })
        });

        if is_featured || !StandardPoolCatalog::contains(pool, item) {
            BannerOutcome::Win
        } else {
            BannerOutcome::Loss
        }
    }
}

/// Central registry of permanent items that count as confirmed off-banner pulls.
pub(crate) struct StandardPoolCatalog;

impl StandardPoolCatalog {
    /// Returns whether an item is permanent for the game represented by `pool`.
    pub(crate) fn contains(pool: PullPool, item: PullItem) -> bool {
        match (pool, item) {
            (PullPool::Hsr(_), PullItem::Character(id)) => HSR_STANDARD_CHARACTERS.contains(&id),
            (PullPool::Hsr(_), PullItem::LightCone(id)) => HSR_STANDARD_LIGHT_CONES.contains(&id),
            (PullPool::Gi(_), PullItem::Character(id)) => GI_STANDARD_CHARACTERS.contains(&id),
            (PullPool::Gi(_), PullItem::Weapon(id)) => GI_STANDARD_WEAPONS.contains(&id),
            (PullPool::Zzz(_), PullItem::Character(id)) => ZZZ_STANDARD_CHARACTERS.contains(&id),
            (PullPool::Zzz(_), PullItem::WEngine(id)) => ZZZ_STANDARD_W_ENGINES.contains(&id),
            _ => false,
        }
    }
}

const HSR_STANDARD_CHARACTERS: &[i32] = &[
    1209, // Yanqing
    1004, // Welt
    1101, // Bronya
    1211, // Bailu
    1104, // Gepard
    1107, // Clara
    1003, // Himeko
];
const HSR_STANDARD_LIGHT_CONES: &[i32] = &[
    23000, // Night on the Milky Way
    23002, // Something Irreplaceable
    23003, // But the Battle Isn't Over
    23004, // In the Name of the World
    23005, // Moment of Victory
    23012, // Sleep Like the Dead
    23013, // Time Waits for No One
];
const GI_STANDARD_CHARACTERS: &[i32] = &[
    10000042, // Keqing
    10000016, // Diluc
    10000003, // Jean
    10000035, // Qiqi
    10000069, // Tighnari
    10000079, // Dehya
    10000109, // Yumemizuki Mizuki
    10000041, // Mona
];
const GI_STANDARD_WEAPONS: &[i32] = &[
    15502, // Amos' Bow
    11501, // Aquila Favonia
    14502, // Lost Prayer to the Sacred Winds
    13505, // Primordial Jade Winged-Spear
    14501, // Skyward Atlas
    15501, // Skyward Harp
    12501, // Skyward Pride
    13502, // Skyward Spine
    12502, // Wolf's Gravestone
];
const ZZZ_STANDARD_CHARACTERS: &[i32] = &[
    1021, // Nekomata
    1041, // Soldier 11
    1101, // Koleda
    1141, // Lycaon
    1181, // Grace
    1211, // Rina
];
const ZZZ_STANDARD_W_ENGINES: &[i32] = &[
    14102, // Steel Cushion
    14104, // The Brimstone
    14110, // Hellfire Gears
    14114, // The Restrained
    14118, // Fusion Compiler
    14121, // Weeping Cradle
];

#[cfg(test)]
mod banner_catalog {
    use super::*;
    use crate::{GachaType, GiGachaType, ZzzGachaType};
    use chrono::TimeZone;

    fn time(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, hour, 0, 0).unwrap()
    }

    /// Permanent items and their banner-backed pools across all three games.
    fn standard_cases() -> Vec<(PullPool, PullItem)> {
        vec![
            (PullPool::Hsr(GachaType::Special), PullItem::Character(1209)),
            (PullPool::Hsr(GachaType::Lc), PullItem::LightCone(23000)),
            (PullPool::Hsr(GachaType::Collab), PullItem::Character(1209)),
            (
                PullPool::Hsr(GachaType::CollabLc),
                PullItem::LightCone(23000),
            ),
            (
                PullPool::Gi(GiGachaType::Character),
                PullItem::Character(10000042),
            ),
            (PullPool::Gi(GiGachaType::Weapon), PullItem::Weapon(15502)),
            (
                PullPool::Gi(GiGachaType::Chronicled),
                PullItem::Character(10000042),
            ),
            (
                PullPool::Gi(GiGachaType::Chronicled),
                PullItem::Weapon(15502),
            ),
            (
                PullPool::Zzz(ZzzGachaType::Special),
                PullItem::Character(1021),
            ),
            (
                PullPool::Zzz(ZzzGachaType::WEngine),
                PullItem::WEngine(14102),
            ),
            (
                PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                PullItem::Character(1021),
            ),
            (
                PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                PullItem::WEngine(14102),
            ),
        ]
    }

    #[test]
    fn empty_catalog_uses_the_same_standard_fallback_for_every_game() {
        let catalog = BannerCatalog::default();

        for (pool, item) in standard_cases() {
            assert_eq!(catalog.classify(pool, item, time(2)), BannerOutcome::Loss);
        }

        for (pool, item) in [
            (PullPool::Hsr(GachaType::Special), PullItem::Character(9001)),
            (
                PullPool::Gi(GiGachaType::Character),
                PullItem::Character(9001),
            ),
            (
                PullPool::Zzz(ZzzGachaType::Special),
                PullItem::Character(9001),
            ),
            (
                PullPool::Zzz(ZzzGachaType::Bangboo),
                PullItem::Bangboo(9001),
            ),
        ] {
            assert_eq!(catalog.classify(pool, item, time(2)), BannerOutcome::Win);
        }
    }

    #[test]
    fn featured_standard_items_override_the_loss_fallback_in_every_pool() {
        let cases = standard_cases();
        let catalog = BannerCatalog::new(cases.iter().map(|(pool, item)| BannerEntry {
            pool: *pool,
            item: *item,
            start: time(1),
            end: time(3),
        }));

        for (pool, item) in cases {
            assert_eq!(catalog.classify(pool, item, time(1)), BannerOutcome::Win);
            assert_eq!(catalog.classify(pool, item, time(2)), BannerOutcome::Win);
            assert_eq!(catalog.classify(pool, item, time(3)), BannerOutcome::Loss);
        }
    }

    #[test]
    fn gaps_and_wrong_pools_fall_back_instead_of_using_unrelated_banner_data() {
        let special = PullPool::Hsr(GachaType::Special);
        let collab = PullPool::Hsr(GachaType::Collab);
        let standard_item = PullItem::Character(1209);
        let catalog = BannerCatalog::new([BannerEntry {
            pool: special,
            item: standard_item,
            start: time(1),
            end: time(3),
        }]);

        assert_eq!(
            catalog.classify(special, standard_item, time(2)),
            BannerOutcome::Win
        );
        assert_eq!(
            catalog.classify(special, standard_item, time(3)),
            BannerOutcome::Loss
        );
        assert_eq!(
            catalog.classify(collab, standard_item, time(2)),
            BannerOutcome::Loss
        );
        assert_eq!(
            catalog.classify(special, PullItem::Character(9001), time(3)),
            BannerOutcome::Win
        );
    }

    #[test]
    fn zzz_banner_validation_rejects_invalid_shapes() {
        let valid = &[(true, Some(2), &[2, 102][..]), (false, None, &[3, 103][..])];
        assert!(validate_banner(time(1), time(2), valid));
        assert!(!validate_banner(time(2), time(1), valid));
        assert!(!validate_banner(
            time(1),
            time(2),
            &[(false, None, &[2, 102][..])]
        ));
        assert!(!validate_banner(
            time(1),
            time(2),
            &[(true, None, &[2, 102][..])]
        ));
        assert!(!validate_banner(
            time(1),
            time(2),
            &[(true, Some(3), &[2, 102][..])]
        ));
    }
}

#[cfg(test)]
mod guarantee_tests {
    use super::*;
    #[test]
    fn every_state_outcome_transition() {
        for first in [BannerOutcome::Win, BannerOutcome::Loss] {
            for second in [BannerOutcome::Win, BannerOutcome::Loss] {
                let mut state = GuaranteeState::default();
                assert_eq!(
                    state.advance(first),
                    if first == BannerOutcome::Win {
                        GuaranteedOutcome::Win
                    } else {
                        GuaranteedOutcome::Loss
                    }
                );
                assert_eq!(
                    state.advance(second),
                    if first == BannerOutcome::Loss {
                        GuaranteedOutcome::GuaranteedWin
                    } else if second == BannerOutcome::Win {
                        GuaranteedOutcome::Win
                    } else {
                        GuaranteedOutcome::Loss
                    }
                );
                assert_eq!(
                    state.armed,
                    first == BannerOutcome::Win && second == BannerOutcome::Loss
                );
            }
        }
    }
    #[test]
    fn unknown_banner_pool_cannot_manufacture_featured_window() {
        let start = DateTime::from_timestamp(1700000000, 0).unwrap();
        let end = start + chrono::Duration::hours(1);
        let catalog = BannerCatalog::from_hsr([database::banners::DbBanner {
            id: 1,
            name: "fixture".into(),
            start,
            end,
            character: Some(1209),
            character_gacha_type: Some(999),
            light_cone: None,
            light_cone_gacha_type: None,
        }]);
        assert_eq!(
            catalog.classify(
                PullPool::Hsr(GachaType::Special),
                PullItem::Character(1209),
                start
            ),
            BannerOutcome::Loss
        );
    }
}
