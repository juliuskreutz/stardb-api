use crate::gacha::imports::PullItem;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// A stored pull with a validated item identity and catalog rarity.
///
/// Localized names remain optional because UIGF permits missing labels. Display
/// endpoints require a name in their fallible conversion instead of inventing one.
pub struct DbSignal {
    pub item: PullItem,
    pub id: i64,
    pub name: Option<String>,
    pub rarity: i32,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}
// SQL LEFT JOINs can yield NULL catalog fields, and historical rows can violate
// the import invariant. Keep the raw columns until TryFrom checks both conditions.
struct RawDbSignal {
    pub id: i64,
    pub character: Option<i32>,
    pub bangboo: Option<i32>,
    pub w_engine: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}
impl TryFrom<RawDbSignal> for DbSignal {
    type Error = anyhow::Error;
    fn try_from(r: RawDbSignal) -> anyhow::Result<Self> {
        // Check exactly one column before choosing an item: coalescing first would
        // hide ambiguous rows. Column identity also supports synthetic numeric IDs.
        let item = match (r.character, r.bangboo, r.w_engine) {
            (Some(id), None, None) => PullItem::Character(id),
            (None, Some(id), None) => PullItem::Bangboo(id),
            (None, None, Some(id)) => PullItem::WEngine(id),
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
pub struct DbSignalInfo {
    pub item: PullItem,
    pub rarity: i32,
    pub timestamp: DateTime<Utc>,
}
struct RawDbSignalInfo {
    pub bangboo: Option<i32>,
    pub character: Option<i32>,
    pub w_engine: Option<i32>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
}
impl TryFrom<RawDbSignalInfo> for DbSignalInfo {
    type Error = anyhow::Error;
    fn try_from(r: RawDbSignalInfo) -> anyhow::Result<Self> {
        let item = match (r.character, r.bangboo, r.w_engine) {
            (Some(id), None, None) => PullItem::Character(id),
            (None, Some(id), None) => PullItem::Bangboo(id),
            (None, None, Some(id)) => PullItem::WEngine(id),
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
    pub bangboo: Vec<Option<i32>>,
    pub w_engine: Vec<Option<i32>>,
    pub timestamp: Vec<DateTime<Utc>>,
    pub official: Vec<bool>,
}

pub async fn get_uids(pool: &PgPool) -> anyhow::Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/zzz/signals/get_uids.sql")
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
        pub async fn set_all(
            set_all: &SetAll,
            connection: &mut PgConnection,
        ) -> anyhow::Result<u64> {
            let result = sqlx::query_file!(
                $sql,
                &set_all.id,
                &set_all.uid,
                &set_all.$item_column as &[Option<i32>],
                &set_all.w_engine as &[Option<i32>],
                &set_all.timestamp as &[DateTime<Utc>],
                &set_all.official,
            )
            .execute(&mut *connection)
            .await?;

            Ok(result.rows_affected())
        }
    };
    (get_earliest_timestamp_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_earliest_timestamp_by_uid(
            uid: i32,
            pool: &PgPool,
        ) -> anyhow::Result<Option<DateTime<Utc>>> {
            Ok(sqlx::query_file!($sql, uid)
                .fetch_one(pool)
                .await?
                .timestamp)
        }
    };
    (get_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_by_uid(
            uid: i32,
            language: Language,
            pool: &PgPool,
        ) -> anyhow::Result<Vec<DbSignal>> {
            let language = language.to_string();
            sqlx::query_file_as!(RawDbSignal, $sql, uid, language)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(DbSignal::try_from)
                .collect()
        }
    };
    (get_infos_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_infos_by_uid<'e, E>(
            uid: i32,
            executor: E,
        ) -> anyhow::Result<Vec<DbSignalInfo>>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
        {
            sqlx::query_file_as!(RawDbSignalInfo, $sql, uid)
                .fetch_all(executor)
                .await?
                .into_iter()
                .map(DbSignalInfo::try_from)
                .collect()
        }
    };
    (delete_all, $sql:literal, $item_column:ident) => {
        pub async fn delete_all(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
            sqlx::query_file!($sql, uid).execute(pool).await?;

            Ok(())
        }
    };
    (delete_unofficial, $sql:literal, $item_column:ident) => {
        pub async fn delete_unofficial(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
            sqlx::query_file!($sql, uid).execute(pool).await?;

            Ok(())
        }
    };
}
// The registry's item column is bangboo only for that pool; all other pools bind
// character followed by w_engine. It is a schema choice, not an ID-range heuristic.
macro_rules! pool_registry {
 ($( $module:ident => $variant:ident, $item:ident { $( $function:ident : $sql:literal ),* $(,)? } ),* $(,)?) => {
$(pub mod $module {
use super::*;
use sqlx::PgConnection;
use crate::Language;
$(pool_fn!($function, $sql, $item);)*
})*
pub async fn set_all_by_pool(kind:crate::ZzzGachaType,set_all: &SetAll, connection: &mut sqlx::PgConnection)->anyhow::Result<u64> { match kind { $(crate::ZzzGachaType::$variant => $module::set_all(set_all, connection).await,)* } }
pub async fn delete_all_by_pool(kind:crate::ZzzGachaType,uid:i32, pool:&PgPool)->anyhow::Result<()> { match kind { $(crate::ZzzGachaType::$variant => $module::delete_all(uid,pool).await,)* } }
pub async fn delete_unofficial_by_pool(kind:crate::ZzzGachaType,uid:i32, pool:&PgPool)->anyhow::Result<()> { match kind { $(crate::ZzzGachaType::$variant => $module::delete_unofficial(uid,pool).await,)* } }
pub async fn get_by_uid_by_pool(kind:crate::ZzzGachaType,uid:i32, language:crate::Language, pool:&PgPool)->anyhow::Result<Vec<DbSignal>> { match kind { $(crate::ZzzGachaType::$variant => $module::get_by_uid(uid,language,pool).await,)* } }
pub async fn get_earliest_timestamp_by_uid_by_pool(kind:crate::ZzzGachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::ZzzGachaType::$variant => $module::get_earliest_timestamp_by_uid(uid,pool).await,)* } }
};
}
pool_registry! {
standard => Standard, character {
set_all: "sql/zzz/signals/standard/set_all.sql",
get_earliest_timestamp_by_uid: "sql/zzz/signals/standard/get_earliest_timestamp_by_uid.sql",
get_by_uid: "sql/zzz/signals/standard/get_by_uid.sql",
get_infos_by_uid: "sql/zzz/signals/standard/get_infos.sql",
delete_all: "sql/zzz/signals/standard/delete_all.sql",
delete_unofficial: "sql/zzz/signals/standard/delete_unofficial.sql",
},
special => Special, character {
set_all: "sql/zzz/signals/special/set_all.sql",
get_earliest_timestamp_by_uid: "sql/zzz/signals/special/get_earliest_timestamp_by_uid.sql",
get_by_uid: "sql/zzz/signals/special/get_by_uid.sql",
get_infos_by_uid: "sql/zzz/signals/special/get_infos.sql",
delete_all: "sql/zzz/signals/special/delete_all.sql",
delete_unofficial: "sql/zzz/signals/special/delete_unofficial.sql",
},
w_engine => WEngine, character {
set_all: "sql/zzz/signals/w_engine/set_all.sql",
get_earliest_timestamp_by_uid: "sql/zzz/signals/w_engine/get_earliest_timestamp_by_uid.sql",
get_by_uid: "sql/zzz/signals/w_engine/get_by_uid.sql",
get_infos_by_uid: "sql/zzz/signals/w_engine/get_infos.sql",
delete_all: "sql/zzz/signals/w_engine/delete_all.sql",
delete_unofficial: "sql/zzz/signals/w_engine/delete_unofficial.sql",
},
bangboo => Bangboo, bangboo {
set_all: "sql/zzz/signals/bangboo/set_all.sql",
get_earliest_timestamp_by_uid: "sql/zzz/signals/bangboo/get_earliest_timestamp_by_uid.sql",
get_by_uid: "sql/zzz/signals/bangboo/get_by_uid.sql",
get_infos_by_uid: "sql/zzz/signals/bangboo/get_infos.sql",
delete_all: "sql/zzz/signals/bangboo/delete_all.sql",
delete_unofficial: "sql/zzz/signals/bangboo/delete_unofficial.sql",
},
exclusive_rescreening => ExclusiveRescreening, character {
set_all: "sql/zzz/signals/exclusive_rescreening/set_all.sql",
get_earliest_timestamp_by_uid: "sql/zzz/signals/exclusive_rescreening/get_earliest_timestamp_by_uid.sql",
get_by_uid: "sql/zzz/signals/exclusive_rescreening/get_by_uid.sql",
get_infos_by_uid: "sql/zzz/signals/exclusive_rescreening/get_infos.sql",
delete_all: "sql/zzz/signals/exclusive_rescreening/delete_all.sql",
delete_unofficial: "sql/zzz/signals/exclusive_rescreening/delete_unofficial.sql",
},
w_engine_reverberation => WEngineReverberation, character {
set_all: "sql/zzz/signals/w_engine_reverberation/set_all.sql",
get_earliest_timestamp_by_uid: "sql/zzz/signals/w_engine_reverberation/get_earliest_timestamp_by_uid.sql",
get_by_uid: "sql/zzz/signals/w_engine_reverberation/get_by_uid.sql",
get_infos_by_uid: "sql/zzz/signals/w_engine_reverberation/get_infos.sql",
delete_all: "sql/zzz/signals/w_engine_reverberation/delete_all.sql",
delete_unofficial: "sql/zzz/signals/w_engine_reverberation/delete_unofficial.sql",
},
}

#[cfg(test)]
mod decode_tests {
    use super::*;
    fn row() -> RawDbSignal {
        RawDbSignal {
            id: 1,
            character: None,
            bangboo: None,
            w_engine: None,
            rarity: Some(5),
            name: Some("fixture".into()),
            timestamp: DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        }
    }
    #[test]
    fn rejects_missing_ambiguous_identity_and_catalog_data() {
        assert!(DbSignal::try_from(row()).is_err());
        let mut r = row();
        r.character = Some(1700000000);
        r.w_engine = Some(2);
        assert!(DbSignal::try_from(r).is_err());
        let mut r = row();
        r.character = Some(1700000000);
        r.rarity = None;
        assert!(DbSignal::try_from(r).is_err());
    }
    #[test]
    fn character_identity_is_explicit() {
        let mut r = row();
        r.character = Some(1700000000);
        assert_eq!(
            DbSignal::try_from(r).unwrap().item,
            PullItem::Character(1700000000)
        );
    }
    #[test]
    fn bangboo_identity_is_explicit() {
        let mut r = row();
        r.bangboo = Some(1700000000);
        assert_eq!(
            DbSignal::try_from(r).unwrap().item,
            PullItem::Bangboo(1700000000)
        );
    }
    #[test]
    fn w_engine_identity_is_explicit() {
        let mut r = row();
        r.w_engine = Some(1700000000);
        assert_eq!(
            DbSignal::try_from(r).unwrap().item,
            PullItem::WEngine(1700000000)
        );
    }
}
