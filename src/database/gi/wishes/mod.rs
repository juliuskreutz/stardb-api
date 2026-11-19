//! Genshin pull persistence with explicit write columns and validated read models.
//! The pool registry preserves public module paths and routes reads/writes by enum.

use crate::gacha::imports::PullItem;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// A stored pull with a validated item identity and catalog rarity.
///
/// Localized names remain optional because UIGF permits missing labels. Display
/// endpoints require a name in their fallible conversion instead of inventing one.
pub struct DbWish {
    pub item: PullItem,
    pub id: i64,
    pub name: Option<String>,
    pub rarity: i32,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}
// SQL LEFT JOINs can yield NULL catalog fields, and historical rows can violate
// the import invariant. Keep the raw columns until TryFrom checks both conditions.
struct RawDbWish {
    pub id: i64,
    pub character: Option<i32>,
    pub weapon: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}
impl TryFrom<RawDbWish> for DbWish {
    type Error = anyhow::Error;
    /// Validates exactly one stored item column and requires catalog rarity before exposing a typed row.
    fn try_from(r: RawDbWish) -> anyhow::Result<Self> {
        // Check exactly one column before choosing an item: coalescing first would
        // hide ambiguous rows. Column identity also supports synthetic numeric IDs.
        let item = match (r.character, r.weapon) {
            (Some(id), None) => PullItem::Character(id),
            (None, Some(id)) => PullItem::Weapon(id),
            _ => anyhow::bail!("invalid stored pull identity: expected exactly one item"),
        };
        Ok(Self {
            timestamp: r.timestamp,
            rarity: r
                .rarity
                .ok_or_else(|| anyhow::anyhow!("missing pull rarity"))?,
            item,
            id: r.id,
            official: r.official,
            name: r.name,
        })
    }
}

/// Label-free stats input; it enforces the same identity/rarity rules as full reads.
pub struct DbWishInfo {
    pub item: PullItem,
    pub rarity: i32,
    pub timestamp: DateTime<Utc>,
}
struct RawDbWishInfo {
    pub character: Option<i32>,
    pub weapon: Option<i32>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
}
impl TryFrom<RawDbWishInfo> for DbWishInfo {
    type Error = anyhow::Error;
    /// Validates exactly one stored item column and requires catalog rarity before exposing a typed row.
    fn try_from(r: RawDbWishInfo) -> anyhow::Result<Self> {
        let item = match (r.character, r.weapon) {
            (Some(id), None) => PullItem::Character(id),
            (None, Some(id)) => PullItem::Weapon(id),
            _ => anyhow::bail!("invalid stored pull identity: expected exactly one item"),
        };
        Ok(Self {
            timestamp: r.timestamp,
            rarity: r
                .rarity
                .ok_or_else(|| anyhow::anyhow!("missing pull rarity"))?,
            item,
        })
    }
}

/// Write arrays retain explicit item columns; item kind must never be inferred
/// from numeric ID ranges when binding these arrays to a pool's SQL parameters.
#[derive(Default)]
pub struct SetAll {
    pub id: Vec<i64>,
    pub uid: Vec<i32>,
    pub character: Vec<Option<i32>>,
    pub weapon: Vec<Option<i32>>,
    pub timestamp: Vec<DateTime<Utc>>,
    pub official: Vec<bool>,
}

/// Returns UIDs with stored pulls and no private connection, for sitemap discovery.
/// The query spans every pool; result order is unspecified.
pub async fn get_uids(pool: &PgPool) -> anyhow::Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/gi/wishes/get_uids.sql")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| r.uid)
        .collect())
}

// SQLx's file macros require literal paths (not concat!), so the registry keeps
// every path explicit while stamping the existing public per-pool function names.
// Keep set_all's item-column order aligned with its SQL parameters: both arrays
// share the same Rust type, so swapping them would compile but corrupt identity.
macro_rules! pool_fn {
    (set_all, $sql:literal, $item_column:ident) => {
        /// Persists normalized parallel arrays through the supplied connection.
        /// Inserts new pulls or upgrades unofficial records to official data; existing
        /// official rows are preserved. Returns affected rows and leaves commit ownership
        /// to the caller. Arrays must have matching lengths and explicit item columns.
        pub async fn set_all(
            set_all: &SetAll,
            connection: &mut PgConnection,
        ) -> anyhow::Result<u64> {
            let result = sqlx::query_file!(
                $sql,
                &set_all.id,
                &set_all.uid,
                &set_all.character as &[Option<i32>],
                &set_all.weapon as &[Option<i32>],
                &set_all.timestamp as &[DateTime<Utc>],
                &set_all.official,
            )
            .execute(&mut *connection)
            .await?;

            Ok(result.rows_affected())
        }
    };
    (get_by_uid, $sql:literal, $item_column:ident) => {
        /// Reads localized history in ascending pull-ID order.
        /// Invalid identity or absent rarity fails the read; nullable names are left
        /// for each consumer to handle according to its output contract.
        pub async fn get_by_uid(
            uid: i32,
            language: Language,
            pool: &PgPool,
        ) -> anyhow::Result<Vec<DbWish>> {
            let language = language.to_string();
            sqlx::query_file_as!(RawDbWish, $sql, uid, language)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(DbWish::try_from)
                .collect()
        }
    };
    (get_infos_by_uid, $sql:literal, $item_column:ident) => {
        /// Reads label-free rows in ascending pull-ID order for stat scans.
        /// Invalid item identity or missing catalog rarity fails the whole read; the
        /// supplied executor may participate in the caller’s import transaction.
        pub async fn get_infos_by_uid<'e, E>(
            uid: i32,
            executor: E,
        ) -> anyhow::Result<Vec<DbWishInfo>>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            sqlx::query_file_as!(RawDbWishInfo, $sql, uid)
                .fetch_all(executor)
                .await?
                .into_iter()
                .map(DbWishInfo::try_from)
                .collect()
        }
    };
    (get_earliest_timestamp_by_uid, $sql:literal, $item_column:ident) => {
        /// Returns the minimum stored timestamp in this pool, or None for an empty history.
        pub async fn get_earliest_timestamp_by_uid(
            uid: i32,
            pool: &PgPool,
        ) -> anyhow::Result<Option<DateTime<Utc>>> {
            Ok(sqlx::query_file!($sql, uid).fetch_one(pool).await?.min)
        }
    };
    (get_latest_timestamp_by_uid, $sql:literal, $item_column:ident) => {
        /// Returns the maximum stored timestamp in this pool, or None for an empty history.
        pub async fn get_latest_timestamp_by_uid(
            uid: i32,
            pool: &PgPool,
        ) -> anyhow::Result<Option<DateTime<Utc>>> {
            Ok(sqlx::query_file!($sql, uid).fetch_one(pool).await?.max)
        }
    };
    (delete_all, $sql:literal, $item_column:ident) => {
        /// Deletes every pull for this UID from this pool, propagating database failures.
        pub async fn delete_all(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
            sqlx::query_file!($sql, uid).execute(pool).await?;

            Ok(())
        }
    };
    (delete_unofficial, $sql:literal, $item_column:ident) => {
        /// Deletes only unofficial pulls for this UID; official provenance is preserved.
        pub async fn delete_unofficial(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
            sqlx::query_file!($sql, uid).execute(pool).await?;

            Ok(())
        }
    };
}
// Enum dispatch is generated from the same entries as the pool modules so a new
// pool cannot acquire working SQL wrappers while being omitted from these routes.
macro_rules! pool_registry {
 ($( $module:ident => $variant:ident, $item:ident { $( $function:ident : $sql:literal ),* $(,)? } ),* $(,)?) => {
$(pub mod $module {
use super::*;
use sqlx::PgConnection;
use crate::Language;
$(pool_fn!($function, $sql, $item);)*
})*
/// Routes a normalized batch to its exact pool without committing the caller’s connection.
/// The selected wrapper preserves official-row precedence and returns affected rows.
pub async fn set_all_by_pool(kind:crate::GiGachaType,set_all: &SetAll, connection: &mut sqlx::PgConnection)->anyhow::Result<u64> { match kind { $(crate::GiGachaType::$variant => $module::set_all(set_all, connection).await,)* } }
/// Routes a localized history read to its exact pool in ascending pull-ID order.
/// Invalid stored identity or missing rarity is an error; labels remain optional.
pub async fn get_by_uid_by_pool(kind:crate::GiGachaType,uid:i32, language:crate::Language, pool:&PgPool)->anyhow::Result<Vec<DbWish>> { match kind { $(crate::GiGachaType::$variant => $module::get_by_uid(uid,language,pool).await,)* } }
/// Routes a minimum-timestamp lookup; returns None when this UID has no pulls in the pool.
pub async fn get_earliest_timestamp_by_uid_by_pool(kind:crate::GiGachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::GiGachaType::$variant => $module::get_earliest_timestamp_by_uid(uid,pool).await,)* } }
/// Routes a maximum-timestamp lookup; returns None when this UID has no pulls in the pool.
pub async fn get_latest_timestamp_by_uid_by_pool(kind:crate::GiGachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::GiGachaType::$variant => $module::get_latest_timestamp_by_uid(uid,pool).await,)* } }
};
}
pool_registry! {
standard => Standard, character {
set_all: "sql/gi/wishes/standard/set_all.sql",
get_by_uid: "sql/gi/wishes/standard/get_by_uid.sql",
get_infos_by_uid: "sql/gi/wishes/standard/get_infos.sql",
get_earliest_timestamp_by_uid: "sql/gi/wishes/standard/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/gi/wishes/standard/get_latest_timestamp_by_uid.sql",
delete_all: "sql/gi/wishes/standard/delete_all.sql",
delete_unofficial: "sql/gi/wishes/standard/delete_unofficial.sql",
},
character => Character, character {
set_all: "sql/gi/wishes/character/set_all.sql",
get_by_uid: "sql/gi/wishes/character/get_by_uid.sql",
get_infos_by_uid: "sql/gi/wishes/character/get_infos.sql",
get_earliest_timestamp_by_uid: "sql/gi/wishes/character/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/gi/wishes/character/get_latest_timestamp_by_uid.sql",
delete_all: "sql/gi/wishes/character/delete_all.sql",
delete_unofficial: "sql/gi/wishes/character/delete_unofficial.sql",
},
weapon => Weapon, character {
set_all: "sql/gi/wishes/weapon/set_all.sql",
get_by_uid: "sql/gi/wishes/weapon/get_by_uid.sql",
get_infos_by_uid: "sql/gi/wishes/weapon/get_infos.sql",
get_earliest_timestamp_by_uid: "sql/gi/wishes/weapon/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/gi/wishes/weapon/get_latest_timestamp_by_uid.sql",
delete_all: "sql/gi/wishes/weapon/delete_all.sql",
delete_unofficial: "sql/gi/wishes/weapon/delete_unofficial.sql",
},
chronicled => Chronicled, character {
set_all: "sql/gi/wishes/chronicled/set_all.sql",
get_by_uid: "sql/gi/wishes/chronicled/get_by_uid.sql",
get_infos_by_uid: "sql/gi/wishes/chronicled/get_infos.sql",
get_earliest_timestamp_by_uid: "sql/gi/wishes/chronicled/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/gi/wishes/chronicled/get_latest_timestamp_by_uid.sql",
delete_all: "sql/gi/wishes/chronicled/delete_all.sql",
delete_unofficial: "sql/gi/wishes/chronicled/delete_unofficial.sql",
},
beginner => Beginner, character {
set_all: "sql/gi/wishes/beginner/set_all.sql",
get_by_uid: "sql/gi/wishes/beginner/get_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/gi/wishes/beginner/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/gi/wishes/beginner/get_latest_timestamp_by_uid.sql",
delete_all: "sql/gi/wishes/beginner/delete_all.sql",
delete_unofficial: "sql/gi/wishes/beginner/delete_unofficial.sql",
},
}

#[cfg(test)]
mod decode_tests {
    use super::*;
    fn row() -> RawDbWish {
        RawDbWish {
            id: 1,
            character: None,
            weapon: None,
            rarity: Some(5),
            name: Some("fixture".into()),
            timestamp: DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        }
    }
    #[test]
    fn rejects_missing_ambiguous_identity_and_catalog_data() {
        assert!(DbWish::try_from(row()).is_err());
        let mut r = row();
        r.character = Some(1700000000);
        r.weapon = Some(2);
        assert!(DbWish::try_from(r).is_err());
        let mut r = row();
        r.character = Some(1700000000);
        r.rarity = None;
        assert!(DbWish::try_from(r).is_err());
    }
    #[test]
    fn character_identity_is_explicit() {
        let mut r = row();
        r.character = Some(1700000000);
        assert_eq!(
            DbWish::try_from(r).unwrap().item,
            PullItem::Character(1700000000)
        );
    }
    #[test]
    fn weapon_identity_is_explicit() {
        let mut r = row();
        r.weapon = Some(1700000000);
        assert_eq!(
            DbWish::try_from(r).unwrap().item,
            PullItem::Weapon(1700000000)
        );
    }
}
