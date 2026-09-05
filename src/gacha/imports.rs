//! Normalized, cross-game pull import validation and persistence.
//!
//! Adapters translate their source formats into [`NormalizedPull`] values.
//! This module then owns deduplication, provenance repair rules, transactional
//! writes, and post-write stat recalculation.

use std::collections::{HashMap, HashSet};
use std::fmt;

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::database;
use crate::{GachaType, GiGachaType, ZzzGachaType};

/// A concrete pull pool, including its game namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PullPool {
    Hsr(GachaType),
    Gi(GiGachaType),
    Zzz(ZzzGachaType),
}

/// A normalized item reference that preserves the source game's item kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PullItem {
    Character(i32),
    LightCone(i32),
    Weapon(i32),
    WEngine(i32),
    Bangboo(i32),
}

impl PullItem {
    /// Returns the stored item ID without inferring or changing its item kind.
    pub(crate) fn id(self) -> i32 {
        match self {
            Self::Character(id)
            | Self::LightCone(id)
            | Self::Weapon(id)
            | Self::WEngine(id)
            | Self::Bangboo(id) => id,
        }
    }
}

/// Whether a pull came from an official endpoint or an alternate source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PullProvenance {
    Official,
    Unofficial,
}

/// One source-independent pull ready for validation and persistence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NormalizedPull {
    pub uid: i32,
    pub id: i64,
    pub pool: PullPool,
    pub item: PullItem,
    pub timestamp: DateTime<Utc>,
    pub provenance: PullProvenance,
}

/// A validated, deterministically ordered set of pulls.
#[derive(Clone, Debug)]
pub(crate) struct ImportBatch {
    pulls: Vec<NormalizedPull>,
    provenance: PullProvenance,
}

/// Closed validation failures safe for adapters to map to client errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ImportValidationError {
    InvalidItemForPool,
    ConflictingDuplicate,
    ProvenanceMismatch,
}

impl fmt::Display for ImportValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidItemForPool => "item is not valid for pull pool",
            Self::ConflictingDuplicate => "duplicate pull records disagree",
            Self::ProvenanceMismatch => "pull provenance does not match batch provenance",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ImportValidationError {}

impl ImportBatch {
    /// Validates, deduplicates, and orders pulls before any database write.
    ///
    /// Authentication and overlap cutoffs belong to each source endpoint. This
    /// layer enforces one provenance for the batch and valid typed item identities.
    /// Exact duplicates collapse. Conflicting records with the same
    /// `(pool, uid, id)` fail the whole batch so input order cannot decide which
    /// value wins.
    pub(crate) fn new(
        pulls: impl IntoIterator<Item = NormalizedPull>,
        provenance: PullProvenance,
    ) -> Result<Self, ImportValidationError> {
        let mut unique = HashMap::new();

        for pull in pulls {
            if pull.provenance != provenance {
                return Err(ImportValidationError::ProvenanceMismatch);
            }
            if !item_is_valid(pull.pool, pull.item) {
                return Err(ImportValidationError::InvalidItemForPool);
            }

            let key = (pull.pool, pull.uid, pull.id);
            match unique.get(&key) {
                Some(existing) if existing != &pull => {
                    return Err(ImportValidationError::ConflictingDuplicate);
                }
                Some(_) => {}
                None => {
                    unique.insert(key, pull);
                }
            }
        }

        let mut pulls: Vec<_> = unique.into_values().collect();
        pulls.sort_by_key(|pull| (pool_order(pull.pool), pull.uid, pull.id));
        Ok(Self { pulls, provenance })
    }

    /// Returns the validated pulls in stable persistence order.
    pub(crate) fn pulls(&self) -> &[NormalizedPull] {
        &self.pulls
    }

    /// Returns each game/UID pair whose stats may need recalculation.
    pub(crate) fn affected_uids(&self) -> HashSet<(PullGame, i32)> {
        self.pulls
            .iter()
            .map(|pull| (PullGame::from(pull.pool), pull.uid))
            .collect()
    }
}

/// Counts changed rows overall and per game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PersistenceSummary {
    pub changed_records: u64,
    pub hsr_changed: u64,
    pub gi_changed: u64,
    pub zzz_changed: u64,
}

/// Persists a validated batch on an existing transaction connection.
///
/// Callers retain transaction ownership. Repository upserts permit only the
/// intentional unofficial-to-official provenance repair.
pub(crate) async fn persist_batch(
    batch: &ImportBatch,
    connection: &mut PgConnection,
) -> anyhow::Result<PersistenceSummary> {
    debug_assert!(batch
        .pulls()
        .iter()
        .all(|pull| pull.provenance == batch.provenance));
    let mut hsr: HashMap<GachaType, database::warps::SetAll> = HashMap::new();
    let mut gi: HashMap<GiGachaType, database::gi::wishes::SetAll> = HashMap::new();
    let mut zzz: HashMap<ZzzGachaType, database::zzz::signals::SetAll> = HashMap::new();

    for pull in batch.pulls() {
        let official = pull.provenance == PullProvenance::Official;
        match pull.pool {
            PullPool::Hsr(pool) => {
                let set = hsr.entry(pool).or_default();
                set.id.push(pull.id);
                set.uid.push(pull.uid);
                set.character.push(match pull.item {
                    PullItem::Character(id) => Some(id),
                    _ => None,
                });
                set.light_cone.push(match pull.item {
                    PullItem::LightCone(id) => Some(id),
                    _ => None,
                });
                set.timestamp.push(pull.timestamp);
                set.official.push(official);
            }
            PullPool::Gi(pool) => {
                let set = gi.entry(pool).or_default();
                set.id.push(pull.id);
                set.uid.push(pull.uid);
                set.character.push(match pull.item {
                    PullItem::Character(id) => Some(id),
                    _ => None,
                });
                set.weapon.push(match pull.item {
                    PullItem::Weapon(id) => Some(id),
                    _ => None,
                });
                set.timestamp.push(pull.timestamp);
                set.official.push(official);
            }
            PullPool::Zzz(pool) => {
                let set = zzz.entry(pool).or_default();
                set.id.push(pull.id);
                set.uid.push(pull.uid);
                set.character.push(match pull.item {
                    PullItem::Character(id) => Some(id),
                    _ => None,
                });
                set.w_engine.push(match pull.item {
                    PullItem::WEngine(id) => Some(id),
                    _ => None,
                });
                set.bangboo.push(match pull.item {
                    PullItem::Bangboo(id) => Some(id),
                    _ => None,
                });
                set.timestamp.push(pull.timestamp);
                set.official.push(official);
            }
        }
    }

    let mut hsr_changed = 0;
    for (pool, set) in hsr {
        hsr_changed += database::warps::set_all_by_pool(pool, &set, &mut *connection).await?;
    }
    let mut gi_changed = 0;
    for (pool, set) in gi {
        gi_changed += database::gi::wishes::set_all_by_pool(pool, &set, &mut *connection).await?;
    }
    let mut zzz_changed = 0;
    for (pool, set) in zzz {
        zzz_changed +=
            database::zzz::signals::set_all_by_pool(pool, &set, &mut *connection).await?;
    }

    Ok(PersistenceSummary {
        changed_records: hsr_changed + gi_changed + zzz_changed,
        hsr_changed,
        gi_changed,
        zzz_changed,
    })
}

/// Persists and recalculates every submitted UID in one transaction.
///
/// A persistence or calculation failure rolls back every game represented by
/// the batch, which is required for multi-game UIGF imports. Recalculation also
/// runs when every pull already exists so reimporting refreshes stats after an
/// administrator changes banner metadata.
pub(crate) async fn persist_batch_in_transaction(
    batch: &ImportBatch,
    pool: &PgPool,
) -> anyhow::Result<PersistenceSummary> {
    let mut transaction = pool.begin().await?;
    let summary = persist_batch(batch, &mut transaction).await?;
    for (game, uid) in batch.affected_uids() {
        match game {
            PullGame::Hsr => {
                crate::gacha::stats::hsr::recalculate_hsr_uid(uid, &mut transaction).await?
            }
            PullGame::Gi => {
                crate::gacha::stats::gi::recalculate_gi_uid(uid, &mut transaction).await?
            }
            PullGame::Zzz => {
                crate::gacha::stats::zzz::recalculate_zzz_uid(uid, &mut transaction).await?
            }
        }
    }
    transaction.commit().await?;
    Ok(summary)
}

/// Regression fixture for the retired column-oriented ZZZ adapter.
#[cfg(test)]
pub(crate) fn normalize_zzz_set(
    pool: ZzzGachaType,
    set: &database::zzz::signals::SetAll,
) -> Result<Vec<NormalizedPull>, ImportValidationError> {
    normalize_columns(
        set.id.len(),
        |index| {
            (
                set.id[index],
                set.uid[index],
                set.timestamp[index],
                set.official[index],
            )
        },
        |index| {
            if [
                set.character[index],
                set.bangboo[index],
                set.w_engine[index],
            ]
            .iter()
            .filter(|item| item.is_some())
            .count()
                != 1
            {
                return Err(ImportValidationError::InvalidItemForPool);
            }
            exactly_one(
                set.character[index]
                    .map(PullItem::Character)
                    .or_else(|| set.bangboo[index].map(PullItem::Bangboo)),
                set.w_engine[index].map(PullItem::WEngine),
            )
        },
        PullPool::Zzz(pool),
    )
}

#[cfg(test)]
fn normalize_columns(
    len: usize,
    row: impl Fn(usize) -> (i64, i32, DateTime<Utc>, bool),
    item: impl Fn(usize) -> Result<PullItem, ImportValidationError>,
    pool: PullPool,
) -> Result<Vec<NormalizedPull>, ImportValidationError> {
    (0..len)
        .map(|index| {
            let (id, uid, timestamp, official) = row(index);
            Ok(NormalizedPull {
                uid,
                id,
                pool,
                item: item(index)?,
                timestamp,
                provenance: if official {
                    PullProvenance::Official
                } else {
                    PullProvenance::Unofficial
                },
            })
        })
        .collect()
}

#[cfg(test)]
fn exactly_one(
    first: Option<PullItem>,
    second: Option<PullItem>,
) -> Result<PullItem, ImportValidationError> {
    match (first, second) {
        (Some(item), None) | (None, Some(item)) => Ok(item),
        _ => Err(ImportValidationError::InvalidItemForPool),
    }
}

/// Game-only identity used to group recalculation work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PullGame {
    Hsr,
    Gi,
    Zzz,
}

impl From<PullPool> for PullGame {
    /// Drops the pool variant while preserving its game namespace.
    fn from(pool: PullPool) -> Self {
        match pool {
            PullPool::Hsr(_) => Self::Hsr,
            PullPool::Gi(_) => Self::Gi,
            PullPool::Zzz(_) => Self::Zzz,
        }
    }
}

/// Checks game and pool item-kind compatibility without consulting numeric ID ranges.
fn item_is_valid(pool: PullPool, item: PullItem) -> bool {
    match pool {
        PullPool::Hsr(GachaType::Lc | GachaType::CollabLc) => {
            matches!(item, PullItem::LightCone(_))
        }
        PullPool::Hsr(_) => matches!(item, PullItem::Character(_) | PullItem::LightCone(_)),
        PullPool::Gi(GiGachaType::Weapon) => matches!(item, PullItem::Weapon(_)),
        PullPool::Gi(_) => matches!(item, PullItem::Character(_) | PullItem::Weapon(_)),
        PullPool::Zzz(ZzzGachaType::Bangboo) => {
            matches!(item, PullItem::Bangboo(_) | PullItem::WEngine(_))
        }
        PullPool::Zzz(ZzzGachaType::WEngine | ZzzGachaType::WEngineReverberation) => {
            matches!(item, PullItem::WEngine(_))
        }
        PullPool::Zzz(_) => matches!(item, PullItem::Character(_) | PullItem::WEngine(_)),
    }
}

/// Assigns a deterministic game-prefixed sort key to each pool.
fn pool_order(pool: PullPool) -> i32 {
    match pool {
        PullPool::Hsr(pool) => pool.id(),
        PullPool::Gi(pool) => 1_000 + pool as i32,
        PullPool::Zzz(pool) => 2_000 + pool.id(),
    }
}

#[cfg(test)]
mod gacha_imports {
    use super::*;
    use chrono::TimeZone;

    fn pull(pool: PullPool, item: PullItem) -> NormalizedPull {
        NormalizedPull {
            uid: 1,
            id: 2,
            pool,
            item,
            timestamp: Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            provenance: PullProvenance::Official,
        }
    }

    fn policy() -> PullProvenance {
        PullProvenance::Official
    }

    #[test]
    fn normalize_accepts_every_pool_and_legal_item_kind() {
        let cases = [
            pull(PullPool::Hsr(GachaType::Standard), PullItem::Character(1)),
            pull(PullPool::Hsr(GachaType::Departure), PullItem::LightCone(1)),
            pull(PullPool::Hsr(GachaType::Special), PullItem::Character(1)),
            pull(PullPool::Hsr(GachaType::Lc), PullItem::LightCone(1)),
            pull(PullPool::Hsr(GachaType::Collab), PullItem::Character(1)),
            pull(PullPool::Hsr(GachaType::CollabLc), PullItem::LightCone(1)),
            pull(PullPool::Gi(GiGachaType::Beginner), PullItem::Character(1)),
            pull(PullPool::Gi(GiGachaType::Standard), PullItem::Weapon(1)),
            pull(PullPool::Gi(GiGachaType::Character), PullItem::Character(1)),
            pull(PullPool::Gi(GiGachaType::Weapon), PullItem::Weapon(1)),
            pull(PullPool::Gi(GiGachaType::Chronicled), PullItem::Weapon(1)),
            pull(PullPool::Zzz(ZzzGachaType::Standard), PullItem::WEngine(1)),
            pull(PullPool::Zzz(ZzzGachaType::Special), PullItem::Character(1)),
            pull(PullPool::Zzz(ZzzGachaType::WEngine), PullItem::WEngine(1)),
            pull(PullPool::Zzz(ZzzGachaType::Bangboo), PullItem::Bangboo(1)),
            pull(
                PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                PullItem::Character(1),
            ),
            pull(
                PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                PullItem::WEngine(1),
            ),
        ];

        for case in cases {
            assert!(ImportBatch::new([case], policy()).is_ok());
        }
    }

    #[test]
    fn normalize_rejects_an_item_from_another_game() {
        let result = ImportBatch::new(
            [pull(
                PullPool::Gi(GiGachaType::Weapon),
                PullItem::LightCone(1),
            )],
            policy(),
        );
        assert_eq!(
            result.unwrap_err(),
            ImportValidationError::InvalidItemForPool
        );
    }

    #[test]
    fn normalize_collapses_exact_duplicates() {
        let value = pull(PullPool::Hsr(GachaType::Standard), PullItem::Character(1));
        let batch = ImportBatch::new([value.clone(), value], policy()).unwrap();
        assert_eq!(batch.pulls().len(), 1);
    }

    #[test]
    fn normalize_rejects_conflicting_duplicates() {
        let first = pull(PullPool::Hsr(GachaType::Standard), PullItem::Character(1));
        let mut second = first.clone();
        second.item = PullItem::Character(2);
        assert_eq!(
            ImportBatch::new([first, second], policy()).unwrap_err(),
            ImportValidationError::ConflictingDuplicate
        );
    }
    #[test]
    fn normalization_rejects_multiple_zzz_items_before_collapse() {
        let mut set = database::zzz::signals::SetAll::default();
        set.id.push(1);
        set.uid.push(1);
        set.character.push(Some(1));
        set.bangboo.push(Some(2));
        set.w_engine.push(None);
        set.timestamp.push(Utc::now());
        set.official.push(false);
        assert_eq!(
            normalize_zzz_set(ZzzGachaType::Standard, &set).unwrap_err(),
            ImportValidationError::InvalidItemForPool
        );
        set.character[0] = None;
        set.bangboo[0] = None;
        assert_eq!(
            normalize_zzz_set(ZzzGachaType::Standard, &set).unwrap_err(),
            ImportValidationError::InvalidItemForPool
        );
    }
    #[test]
    fn mixed_provenance_rejected() {
        let first = pull(PullPool::Hsr(GachaType::Standard), PullItem::Character(1));
        let mut other = first.clone();
        other.id += 1;
        other.provenance = PullProvenance::Unofficial;
        assert_eq!(
            ImportBatch::new([first, other], PullProvenance::Official).unwrap_err(),
            ImportValidationError::ProvenanceMismatch
        );
    }
}

#[cfg(test)]
mod gacha_import_db {
    mod provenance {
        use super::super::*;
        use chrono::{Duration, TimeZone};
        use sqlx::postgres::PgPoolOptions;
        use uuid::Uuid;

        #[actix_web::test]
        async fn only_official_data_repairs_an_unofficial_pull_in_all_17_pools() {
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .expect("test database connects");
            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("database migrates");
            let mut transaction = pool.begin().await.expect("transaction starts");

            let suffix = (Uuid::new_v4().as_u128() % 100_000_000) as i32;
            let uid = 1_600_000_000 + suffix;
            let first_item = 1_700_000_000 + suffix;
            let repaired_item = first_item + 1;
            sqlx::query("INSERT INTO mihomo (uid, region, name, level, signature, avatar_icon, achievement_count) VALUES ($1, 'na', 'test', 1, '', '', 0)")
                .bind(uid).execute(&mut *transaction).await.unwrap();
            sqlx::query("INSERT INTO gi_profiles (uid, name) VALUES ($1, 'test')")
                .bind(uid)
                .execute(&mut *transaction)
                .await
                .unwrap();
            sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1)")
                .bind(uid)
                .execute(&mut *transaction)
                .await
                .unwrap();
            for table in [
                "characters",
                "light_cones",
                "gi_characters",
                "gi_weapons",
                "zzz_characters",
                "zzz_w_engines",
                "zzz_bangboos",
            ] {
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "INSERT INTO {table} (id, rarity) VALUES ($1, 5), ($2, 5)"
                )))
                .bind(first_item)
                .bind(repaired_item)
                .execute(&mut *transaction)
                .await
                .unwrap();
            }

            let timestamp = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
            let games = [
                (
                    PullPool::Hsr(GachaType::Standard),
                    "warps_standard",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Hsr(GachaType::Departure),
                    "warps_departure",
                    "light_cone",
                    PullItem::LightCone(repaired_item),
                ),
                (
                    PullPool::Hsr(GachaType::Special),
                    "warps_special",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Hsr(GachaType::Lc),
                    "warps_lc",
                    "light_cone",
                    PullItem::LightCone(repaired_item),
                ),
                (
                    PullPool::Hsr(GachaType::Collab),
                    "warps_collab",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Hsr(GachaType::CollabLc),
                    "warps_collab_lc",
                    "light_cone",
                    PullItem::LightCone(repaired_item),
                ),
                (
                    PullPool::Gi(GiGachaType::Beginner),
                    "gi_wishes_beginner",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Gi(GiGachaType::Standard),
                    "gi_wishes_standard",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Gi(GiGachaType::Character),
                    "gi_wishes_character",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Gi(GiGachaType::Weapon),
                    "gi_wishes_weapon",
                    "weapon",
                    PullItem::Weapon(repaired_item),
                ),
                (
                    PullPool::Gi(GiGachaType::Chronicled),
                    "gi_wishes_chronicled",
                    "weapon",
                    PullItem::Weapon(repaired_item),
                ),
                (
                    PullPool::Zzz(ZzzGachaType::Standard),
                    "zzz_signals_standard",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Zzz(ZzzGachaType::Special),
                    "zzz_signals_special",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Zzz(ZzzGachaType::WEngine),
                    "zzz_signals_w_engine",
                    "w_engine",
                    PullItem::WEngine(repaired_item),
                ),
                (
                    PullPool::Zzz(ZzzGachaType::Bangboo),
                    "zzz_signals_bangboo",
                    "bangboo",
                    PullItem::Bangboo(repaired_item),
                ),
                (
                    PullPool::Zzz(ZzzGachaType::ExclusiveRescreening),
                    "zzz_signals_exclusive_rescreening",
                    "character",
                    PullItem::Character(repaired_item),
                ),
                (
                    PullPool::Zzz(ZzzGachaType::WEngineReverberation),
                    "zzz_signals_w_engine_reverberation",
                    "w_engine",
                    PullItem::WEngine(repaired_item),
                ),
            ];
            assert_eq!(games.len(), 17);
            for (game_index, (pull_pool, table, column, repaired_pull_item)) in
                games.into_iter().enumerate()
            {
                for transition in 0..4_i64 {
                    let id = 9_000_000_000 + game_index as i64 * 10 + transition;
                    let stored_official = matches!(transition, 1 | 2);
                    let incoming_official = matches!(transition, 0 | 2);
                    sqlx::query(sqlx::AssertSqlSafe(format!("INSERT INTO {table} (id, uid, {column}, timestamp, official) VALUES ($1, $2, $3, $4, $5)")))
                        .bind(id).bind(uid).bind(first_item).bind(timestamp).bind(stored_official)
                        .execute(&mut *transaction).await.unwrap();

                    let provenance = if incoming_official {
                        PullProvenance::Official
                    } else {
                        PullProvenance::Unofficial
                    };
                    let batch = ImportBatch::new(
                        [NormalizedPull {
                            uid,
                            id,
                            pool: pull_pool,
                            item: repaired_pull_item,
                            timestamp: timestamp + Duration::hours(1),
                            provenance,
                        }],
                        provenance,
                    )
                    .unwrap();
                    let summary = persist_batch(&batch, &mut transaction).await.unwrap();
                    let expected_repair = !stored_official && incoming_official;
                    assert_eq!(summary.changed_records, u64::from(expected_repair));

                    let row: (i32, bool) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
                        "SELECT {column}, official FROM {table} WHERE uid = $1 AND id = $2"
                    )))
                    .bind(uid)
                    .bind(id)
                    .fetch_one(&mut *transaction)
                    .await
                    .unwrap();
                    assert_eq!(
                        row,
                        (
                            if expected_repair {
                                repaired_item
                            } else {
                                first_item
                            },
                            stored_official || incoming_official
                        )
                    );
                }
            }

            transaction.rollback().await.unwrap();
        }
    }

    mod uigf_atomic {
        use super::super::*;
        use chrono::TimeZone;
        use sqlx::postgres::PgPoolOptions;
        use uuid::Uuid;

        #[actix_web::test]
        async fn multi_game_failure_rolls_back_and_success_commits_current_stats() {
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
            let pool = PgPoolOptions::new()
                .max_connections(2)
                .connect(&database_url)
                .await
                .unwrap();
            sqlx::migrate!().run(&pool).await.unwrap();

            let suffix = (Uuid::new_v4().as_u128() % 100_000_000) as i32;
            let uid = 1_700_000_000 + suffix;
            let item = 1_800_000_000 + suffix;
            sqlx::query("INSERT INTO mihomo (uid, region, name, level, signature, avatar_icon, achievement_count) VALUES ($1, 'na', 'test', 1, '', '', 0)")
                .bind(uid).execute(&pool).await.unwrap();
            sqlx::query("INSERT INTO gi_profiles (uid, name) VALUES ($1, 'test')")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1)")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            for table in [
                "characters",
                "light_cones",
                "gi_characters",
                "gi_weapons",
                "zzz_characters",
                "zzz_w_engines",
                "zzz_bangboos",
            ] {
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "INSERT INTO {table} (id, rarity) VALUES ($1, 5)"
                )))
                .bind(item)
                .execute(&pool)
                .await
                .unwrap();
            }

            let timestamp = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
            let policy = PullProvenance::Unofficial;
            let invalid = ImportBatch::new(
                [
                    NormalizedPull {
                        uid,
                        id: 10,
                        pool: PullPool::Hsr(GachaType::Standard),
                        item: PullItem::Character(item),
                        timestamp,
                        provenance: PullProvenance::Unofficial,
                    },
                    NormalizedPull {
                        uid,
                        id: 11,
                        pool: PullPool::Gi(GiGachaType::Standard),
                        item: PullItem::Character(item + 1),
                        timestamp,
                        provenance: PullProvenance::Unofficial,
                    },
                ],
                policy,
            )
            .unwrap();
            assert!(persist_batch_in_transaction(&invalid, &pool).await.is_err());
            let hsr_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM warps_standard WHERE uid = $1")
                    .bind(uid)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(hsr_count, 0);

            let valid = ImportBatch::new(
                [
                    NormalizedPull {
                        uid,
                        id: 20,
                        pool: PullPool::Hsr(GachaType::Standard),
                        item: PullItem::Character(item),
                        timestamp,
                        provenance: PullProvenance::Unofficial,
                    },
                    NormalizedPull {
                        uid,
                        id: 21,
                        pool: PullPool::Gi(GiGachaType::Standard),
                        item: PullItem::Character(item),
                        timestamp,
                        provenance: PullProvenance::Unofficial,
                    },
                    NormalizedPull {
                        uid,
                        id: 22,
                        pool: PullPool::Zzz(ZzzGachaType::Standard),
                        item: PullItem::Character(item),
                        timestamp,
                        provenance: PullProvenance::Unofficial,
                    },
                ],
                policy,
            )
            .unwrap();
            assert_eq!(
                persist_batch_in_transaction(&valid, &pool)
                    .await
                    .unwrap()
                    .changed_records,
                3
            );

            for table in [
                "warps_stats_standard",
                "gi_wishes_stats_standard",
                "zzz_signals_stats_standard",
            ] {
                let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT COUNT(*) FROM {table} WHERE uid = $1"
                )))
                .bind(uid)
                .fetch_one(&pool)
                .await
                .unwrap();
                assert_eq!(count, 1, "{table} is current before return");
            }

            for table in [
                "warps_stats_standard",
                "gi_wishes_stats_standard",
                "zzz_signals_stats_standard",
            ] {
                sqlx::query(sqlx::AssertSqlSafe(format!(
                    "DELETE FROM {table} WHERE uid = $1"
                )))
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            }

            assert_eq!(
                persist_batch_in_transaction(&valid, &pool)
                    .await
                    .unwrap()
                    .changed_records,
                0,
                "an identical reimport should still recalculate stats"
            );

            for table in [
                "warps_stats_standard",
                "gi_wishes_stats_standard",
                "zzz_signals_stats_standard",
            ] {
                let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
                    "SELECT COUNT(*) FROM {table} WHERE uid = $1"
                )))
                .bind(uid)
                .fetch_one(&pool)
                .await
                .unwrap();
                assert_eq!(count, 1, "{table} is refreshed by an identical reimport");
            }

            sqlx::query("DELETE FROM mihomo WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM gi_profiles WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM zzz_uids WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM characters WHERE id = $1")
                .bind(item)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM gi_characters WHERE id = $1")
                .bind(item)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM zzz_characters WHERE id = $1")
                .bind(item)
                .execute(&pool)
                .await
                .unwrap();
        }
    }
}

#[cfg(test)]
mod new_pool_uid_tests {
    use super::*;
    #[actix_web::test]
    async fn uid_with_only_a_2026_pool_is_included() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let suffix = (uuid::Uuid::new_v4().as_u128() % 100_000_000) as i32;
        let item = 1_900_000_000 + suffix;
        for table in ["zzz_characters", "zzz_w_engines"] {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "INSERT INTO {table} (id, rarity) VALUES ($1, 4)"
            )))
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        }
        for (offset, (table, column)) in [
            ("zzz_signals_exclusive_rescreening", "character"),
            ("zzz_signals_w_engine_reverberation", "w_engine"),
        ]
        .into_iter()
        .enumerate()
        {
            let uid = 1_200_000_000 + suffix + offset as i32;
            sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1)")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query(sqlx::AssertSqlSafe(format!("INSERT INTO {table} (id, uid, {column}, timestamp, official) VALUES (1, $1, $2, NOW(), false)"))).bind(uid).bind(item).execute(&pool).await.unwrap();
            assert!(database::zzz::signals::get_uids(&pool)
                .await
                .unwrap()
                .contains(&uid));
            sqlx::query("DELETE FROM zzz_uids WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .unwrap();
        }
        for table in ["zzz_characters", "zzz_w_engines"] {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "DELETE FROM {table} WHERE id = $1"
            )))
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        }
    }
}
