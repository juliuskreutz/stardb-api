use crate::gacha::imports::PullItem;

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::Language;

/// A stored pull with a validated item identity and catalog rarity.
///
/// Localized names remain optional because UIGF permits missing labels. Display
/// endpoints require a name in their fallible conversion instead of inventing one.
pub struct DbWarp {
    pub item: PullItem,
    pub id: i64,
    pub name: Option<String>,
    pub rarity: i32,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}
// SQL LEFT JOINs can yield NULL catalog fields, and historical rows can violate
// the import invariant. Keep the raw columns until TryFrom checks both conditions.
struct RawDbWarp {
    pub id: i64,
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub name: Option<String>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub official: bool,
}
impl TryFrom<RawDbWarp> for DbWarp {
    type Error = anyhow::Error;
    fn try_from(r: RawDbWarp) -> anyhow::Result<Self> {
        // Check exactly one column before choosing an item: coalescing first would
        // hide ambiguous rows. Column identity also supports synthetic numeric IDs.
        let item = match (r.character, r.light_cone) {
            (Some(id), None) => PullItem::Character(id),
            (None, Some(id)) => PullItem::LightCone(id),
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
pub struct DbWarpInfo {
    pub item: PullItem,
    pub rarity: i32,
    pub timestamp: DateTime<Utc>,
}
struct RawDbWarpInfo {
    pub character: Option<i32>,
    pub light_cone: Option<i32>,
    pub rarity: Option<i32>,
    pub timestamp: DateTime<Utc>,
}
impl TryFrom<RawDbWarpInfo> for DbWarpInfo {
    type Error = anyhow::Error;
    fn try_from(r: RawDbWarpInfo) -> anyhow::Result<Self> {
        let item = match (r.character, r.light_cone) {
            (Some(id), None) => PullItem::Character(id),
            (None, Some(id)) => PullItem::LightCone(id),
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
    pub light_cone: Vec<Option<i32>>,
    pub timestamp: Vec<DateTime<Utc>>,
    pub official: Vec<bool>,
}

pub async fn get_uids(pool: &PgPool) -> anyhow::Result<Vec<i32>> {
    Ok(sqlx::query_file!("sql/warps/get_uids.sql")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| r.uid)
        .collect())
}

pub struct DbCharacterCount {
    pub id: i32,
    pub rarity: i32,
    pub name: Option<String>,
    pub path: String,
    pub element: String,
    pub path_id: String,
    pub element_id: String,
    pub count: Option<i64>,
}

pub async fn get_characters_count_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbCharacterCount>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbCharacterCount,
        "sql/warps/get_characters_count_by_uid.sql",
        uid,
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub struct DbLightConeCount {
    pub id: i32,
    pub rarity: i32,
    pub name: Option<String>,
    pub path: String,
    pub path_id: String,
    pub count: Option<i64>,
}

pub async fn get_light_cones_count_by_uid(
    uid: i32,
    language: Language,
    pool: &PgPool,
) -> Result<Vec<DbLightConeCount>> {
    let language = language.to_string();

    Ok(sqlx::query_file_as!(
        DbLightConeCount,
        "sql/warps/get_light_cones_count_by_uid.sql",
        uid,
        language,
    )
    .fetch_all(pool)
    .await?)
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
                &set_all.character as &[Option<i32>],
                &set_all.light_cone as &[Option<i32>],
                &set_all.timestamp as &[DateTime<Utc>],
                &set_all.official,
            )
            .execute(&mut *connection)
            .await?;

            Ok(result.rows_affected())
        }
    };
    (get_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_by_uid(
            uid: i32,
            language: Language,
            pool: &PgPool,
        ) -> anyhow::Result<Vec<DbWarp>> {
            let language = language.to_string();
            sqlx::query_file_as!(RawDbWarp, $sql, uid, language)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(DbWarp::try_from)
                .collect()
        }
    };
    (get_infos_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_infos_by_uid<'e, E>(
            uid: i32,
            executor: E,
        ) -> anyhow::Result<Vec<DbWarpInfo>>
        where
            E: Executor<'e, Database = Postgres>,
        {
            sqlx::query_file_as!(RawDbWarpInfo, $sql, uid)
                .fetch_all(executor)
                .await?
                .into_iter()
                .map(DbWarpInfo::try_from)
                .collect()
        }
    };
    (get_count_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_count_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<i64> {
            Ok(sqlx::query_file!($sql, uid)
                .fetch_one(pool)
                .await?
                .count
                .unwrap())
        }
    };
    (get_earliest_timestamp_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_earliest_timestamp_by_uid(
            uid: i32,
            pool: &PgPool,
        ) -> anyhow::Result<Option<DateTime<Utc>>> {
            Ok(sqlx::query_file!($sql, uid).fetch_one(pool).await?.min)
        }
    };
    (get_latest_timestamp_by_uid, $sql:literal, $item_column:ident) => {
        pub async fn get_latest_timestamp_by_uid(
            uid: i32,
            pool: &PgPool,
        ) -> anyhow::Result<Option<DateTime<Utc>>> {
            Ok(sqlx::query_file!($sql, uid).fetch_one(pool).await?.max)
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
// Enum dispatch is generated from the same entries as the pool modules so a new
// pool cannot acquire working SQL wrappers while being omitted from these routes.
macro_rules! pool_registry {
 ($( $module:ident => $variant:ident, $item:ident { $( $function:ident : $sql:literal ),* $(,)? } ),* $(,)?) => {
$(pub mod $module {
use super::*;
use sqlx::{Executor, Postgres, PgConnection};
use crate::Language;
$(pool_fn!($function, $sql, $item);)*
})*
pub async fn set_all_by_pool(kind:crate::GachaType,set_all: &SetAll, connection: &mut sqlx::PgConnection)->anyhow::Result<u64> { match kind { $(crate::GachaType::$variant => $module::set_all(set_all, connection).await,)* } }
pub async fn delete_all_by_pool(kind:crate::GachaType,uid:i32, pool:&PgPool)->anyhow::Result<()> { match kind { $(crate::GachaType::$variant => $module::delete_all(uid,pool).await,)* } }
pub async fn delete_unofficial_by_pool(kind:crate::GachaType,uid:i32, pool:&PgPool)->anyhow::Result<()> { match kind { $(crate::GachaType::$variant => $module::delete_unofficial(uid,pool).await,)* } }
pub async fn get_by_uid_by_pool(kind:crate::GachaType,uid:i32, language:crate::Language, pool:&PgPool)->anyhow::Result<Vec<DbWarp>> { match kind { $(crate::GachaType::$variant => $module::get_by_uid(uid,language,pool).await,)* } }
pub async fn get_earliest_timestamp_by_uid_by_pool(kind:crate::GachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::GachaType::$variant => $module::get_earliest_timestamp_by_uid(uid,pool).await,)* } }
pub async fn get_latest_timestamp_by_uid_by_pool(kind:crate::GachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::GachaType::$variant => $module::get_latest_timestamp_by_uid(uid,pool).await,)* } }
pub async fn get_count_by_uid_by_pool(kind:crate::GachaType,uid:i32,pool:&PgPool)->anyhow::Result<i64> { match kind { $(crate::GachaType::$variant => $module::get_count_by_uid(uid,pool).await,)* } }
};
}
pool_registry! {
standard => Standard, character {
set_all: "sql/warps/standard/set_all.sql",
get_by_uid: "sql/warps/standard/get_by_uid.sql",
get_infos_by_uid: "sql/warps/standard/get_infos.sql",
get_count_by_uid: "sql/warps/standard/get_count_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/warps/standard/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/warps/standard/get_latest_timestamp_by_uid.sql",
delete_all: "sql/warps/standard/delete_all.sql",
delete_unofficial: "sql/warps/standard/delete_unofficial.sql",
},
special => Special, character {
set_all: "sql/warps/special/set_all.sql",
get_by_uid: "sql/warps/special/get_by_uid.sql",
get_infos_by_uid: "sql/warps/special/get_infos.sql",
get_count_by_uid: "sql/warps/special/get_count_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/warps/special/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/warps/special/get_latest_timestamp_by_uid.sql",
delete_all: "sql/warps/special/delete_all.sql",
delete_unofficial: "sql/warps/special/delete_unofficial.sql",
},
lc => Lc, character {
set_all: "sql/warps/lc/set_all.sql",
get_by_uid: "sql/warps/lc/get_by_uid.sql",
get_infos_by_uid: "sql/warps/lc/get_infos.sql",
get_count_by_uid: "sql/warps/lc/get_count_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/warps/lc/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/warps/lc/get_latest_timestamp_by_uid.sql",
delete_all: "sql/warps/lc/delete_all.sql",
delete_unofficial: "sql/warps/lc/delete_unofficial.sql",
},
collab => Collab, character {
set_all: "sql/warps/collab/set_all.sql",
get_by_uid: "sql/warps/collab/get_by_uid.sql",
get_infos_by_uid: "sql/warps/collab/get_infos.sql",
get_count_by_uid: "sql/warps/collab/get_count_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/warps/collab/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/warps/collab/get_latest_timestamp_by_uid.sql",
delete_all: "sql/warps/collab/delete_all.sql",
delete_unofficial: "sql/warps/collab/delete_unofficial.sql",
},
collab_lc => CollabLc, character {
set_all: "sql/warps/collab_lc/set_all.sql",
get_by_uid: "sql/warps/collab_lc/get_by_uid.sql",
get_infos_by_uid: "sql/warps/collab_lc/get_infos.sql",
get_count_by_uid: "sql/warps/collab_lc/get_count_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/warps/collab_lc/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/warps/collab_lc/get_latest_timestamp_by_uid.sql",
delete_all: "sql/warps/collab_lc/delete_all.sql",
delete_unofficial: "sql/warps/collab_lc/delete_unofficial.sql",
},
departure => Departure, character {
set_all: "sql/warps/departure/set_all.sql",
get_by_uid: "sql/warps/departure/get_by_uid.sql",
get_count_by_uid: "sql/warps/departure/get_count_by_uid.sql",
get_earliest_timestamp_by_uid: "sql/warps/departure/get_earliest_timestamp_by_uid.sql",
get_latest_timestamp_by_uid: "sql/warps/departure/get_latest_timestamp_by_uid.sql",
delete_all: "sql/warps/departure/delete_all.sql",
delete_unofficial: "sql/warps/departure/delete_unofficial.sql",
},
}

#[cfg(test)]
mod decode_tests {
    use super::*;
    fn row() -> RawDbWarp {
        RawDbWarp {
            id: 1,
            character: None,
            light_cone: None,
            rarity: Some(5),
            name: Some("fixture".into()),
            timestamp: DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        }
    }
    #[test]
    fn rejects_missing_ambiguous_identity_and_catalog_data() {
        assert!(DbWarp::try_from(row()).is_err());
        let mut r = row();
        r.character = Some(1700000000);
        r.light_cone = Some(2);
        assert!(DbWarp::try_from(r).is_err());
        let mut r = row();
        r.character = Some(1700000000);
        r.rarity = None;
        assert!(DbWarp::try_from(r).is_err());
    }
    #[test]
    fn character_identity_is_explicit() {
        let mut r = row();
        r.character = Some(1700000000);
        assert_eq!(
            DbWarp::try_from(r).unwrap().item,
            PullItem::Character(1700000000)
        );
    }
    #[test]
    fn light_cone_identity_is_explicit() {
        let mut r = row();
        r.light_cone = Some(1700000000);
        assert_eq!(
            DbWarp::try_from(r).unwrap().item,
            PullItem::LightCone(1700000000)
        );
    }
}
