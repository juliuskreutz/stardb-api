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

/// Classification of a high-rarity pull against known banner coverage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BannerOutcome {
    /// The pulled item is featured in the same pool and time range.
    Featured,
    /// The pulled item belongs to that game's permanent standard pool.
    OffBanner,
    /// No authoritative catalog entry can classify the pull.
    Unknown,
}

impl BannerOutcome {
    /// Converts known outcomes to the legacy boolean representation.
    ///
    /// `Unknown` remains `None` so incomplete history never changes win-rate,
    /// streak, or guarantee state.
    pub(crate) fn as_win(self) -> Option<bool> {
        match self {
            Self::Featured => Some(true),
            Self::OffBanner => Some(false),
            Self::Unknown => None,
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
    pub(crate) fn from_hsr(banners: impl IntoIterator<Item = database::banners::DbBanner>) -> Self {
        Self::new(banners.into_iter().flat_map(|banner| {
            let mut entries = Vec::with_capacity(2);
            if let (Some(item), Some(pool)) = (banner.character, banner.character_gacha_type) {
                entries.push(BannerEntry {
                    pool: PullPool::Hsr(if pool == 21 {
                        GachaType::Collab
                    } else {
                        GachaType::Special
                    }),
                    item: PullItem::Character(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            if let (Some(item), Some(pool)) = (banner.light_cone, banner.light_cone_gacha_type) {
                entries.push(BannerEntry {
                    pool: PullPool::Hsr(if pool == 22 {
                        GachaType::CollabLc
                    } else {
                        GachaType::Lc
                    }),
                    item: PullItem::LightCone(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            entries
        }))
    }

    /// Adapts Genshin persistence rows into Character, Weapon, and Chronicled entries.
    pub(crate) fn from_gi(
        banners: impl IntoIterator<Item = database::gi::banners::DbBanner>,
    ) -> Self {
        Self::new(banners.into_iter().flat_map(|banner| {
            let mut entries = Vec::with_capacity(2);
            if let (Some(item), Some(pool)) = (banner.character, banner.character_gacha_type) {
                entries.push(BannerEntry {
                    pool: PullPool::Gi(if pool == 500 {
                        GiGachaType::Chronicled
                    } else {
                        GiGachaType::Character
                    }),
                    item: PullItem::Character(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            if let (Some(item), Some(pool)) = (banner.weapon, banner.weapon_gacha_type) {
                entries.push(BannerEntry {
                    pool: PullPool::Gi(if pool == 500 {
                        GiGachaType::Chronicled
                    } else {
                        GiGachaType::Weapon
                    }),
                    item: PullItem::Weapon(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            entries
        }))
    }

    /// Adapts ZZZ persistence rows into its five banner-backed pool catalogs.
    pub(crate) fn from_zzz(
        banners: impl IntoIterator<Item = database::zzz::banners::DbBanner>,
    ) -> Self {
        Self::new(banners.into_iter().flat_map(|banner| {
            let mut entries = Vec::with_capacity(3);
            if let (Some(item), Some(pool)) = (banner.character, banner.character_gacha_type) {
                entries.push(BannerEntry {
                    pool: PullPool::Zzz(if pool == 102 {
                        ZzzGachaType::ExclusiveRescreening
                    } else {
                        ZzzGachaType::Special
                    }),
                    item: PullItem::Character(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            if let (Some(item), Some(pool)) = (banner.w_engine, banner.w_engine_gacha_type) {
                entries.push(BannerEntry {
                    pool: PullPool::Zzz(if pool == 103 {
                        ZzzGachaType::WEngineReverberation
                    } else {
                        ZzzGachaType::WEngine
                    }),
                    item: PullItem::WEngine(item),
                    start: banner.start,
                    end: banner.end,
                });
            }
            if let (Some(item), Some(5)) = (banner.bangboo, banner.bangboo_gacha_type) {
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

    /// Classifies one pull without guessing across missing coverage.
    ///
    /// Featured ranges are `[start, end)`. Permanent items are known
    /// off-banner results; any other item without an exact entry is unknown.
    pub(crate) fn classify(
        &self,
        pool: PullPool,
        item: PullItem,
        timestamp: DateTime<Utc>,
    ) -> BannerOutcome {
        if StandardPoolCatalog::contains(pool, item) {
            return BannerOutcome::OffBanner;
        }

        self.entries_by_pool
            .get(&pool)
            .and_then(|entries| {
                entries.iter().find(|entry| {
                    entry.item == item && entry.start <= timestamp && timestamp < entry.end
                })
            })
            .map_or(BannerOutcome::Unknown, |_| BannerOutcome::Featured)
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

    #[test]
    fn classifies_by_pool_coverage_and_half_open_range() {
        let hsr_pool = PullPool::Hsr(GachaType::Special);
        let gi_pool = PullPool::Gi(GiGachaType::Character);
        let zzz_pool = PullPool::Zzz(ZzzGachaType::Special);
        let catalog = BannerCatalog::new([
            BannerEntry {
                pool: hsr_pool,
                item: PullItem::Character(9001),
                start: time(1),
                end: time(3),
            },
            BannerEntry {
                pool: hsr_pool,
                item: PullItem::Character(9002),
                start: time(2),
                end: time(4),
            },
            BannerEntry {
                pool: gi_pool,
                item: PullItem::Character(9001),
                start: time(1),
                end: time(3),
            },
            BannerEntry {
                pool: zzz_pool,
                item: PullItem::Character(9001),
                start: time(1),
                end: time(3),
            },
        ]);

        assert_eq!(
            catalog.classify(hsr_pool, PullItem::Character(9001), time(1)),
            BannerOutcome::Featured
        );
        assert_eq!(
            catalog.classify(hsr_pool, PullItem::Character(9002), time(2)),
            BannerOutcome::Featured
        );
        assert_eq!(
            catalog.classify(hsr_pool, PullItem::Character(9001), time(3)),
            BannerOutcome::Unknown
        );
        assert_eq!(
            catalog.classify(
                PullPool::Hsr(GachaType::Collab),
                PullItem::Character(9001),
                time(2)
            ),
            BannerOutcome::Unknown
        );
        assert_eq!(
            catalog.classify(hsr_pool, PullItem::Character(9999), time(2)),
            BannerOutcome::Unknown
        );
        assert_eq!(
            catalog.classify(hsr_pool, PullItem::Character(1209), time(2)),
            BannerOutcome::OffBanner
        );
        assert_eq!(
            catalog.classify(gi_pool, PullItem::Character(1209), time(2)),
            BannerOutcome::Unknown
        );
        assert_eq!(
            catalog.classify(zzz_pool, PullItem::Character(1021), time(2)),
            BannerOutcome::OffBanner
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
