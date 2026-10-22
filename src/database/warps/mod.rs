//! HSR pull persistence with explicit write columns and validated read models.
//! The pool registry preserves public module paths and routes reads/writes by enum.

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
    /// Validates exactly one stored item column and requires catalog rarity before exposing a typed row.
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
    /// Validates exactly one stored item column and requires catalog rarity before exposing a typed row.
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

/// Returns UIDs with stored pulls and no private connection, for sitemap discovery.
/// The query spans every pool; result order is unspecified.
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
    pub name: String,
    pub path: String,
    pub element: String,
    pub path_id: String,
    pub element_id: String,
    pub count: Option<i64>,
}

/// Counts character copies across all HSR pools, including collab, with localized labels.
/// Rows are ordered by rarity descending, then item ID descending.
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
    pub name: String,
    pub path: String,
    pub path_id: String,
    pub count: Option<i64>,
}

/// Counts Light Cone copies across all HSR pools, including collab, with localized labels.
/// Rows are ordered by rarity descending, then item ID descending.
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
        /// Reads localized history in ascending pull-ID order.
        /// Invalid identity or absent rarity fails the read; nullable names are left
        /// for each consumer to handle according to its output contract.
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
        /// Reads label-free rows in ascending pull-ID order for stat scans.
        /// Invalid item identity or missing catalog rarity fails the whole read; the
        /// supplied executor may participate in the caller’s import transaction.
        pub async fn get_infos_by_uid<'e, E>(
            uid: i32,
            executor: E,
        ) -> anyhow::Result<Vec<DbWarpInfo>>
        where
            E: sqlx::Executor<'e, Database = sqlx::Postgres>,
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
        /// Counts all pulls for this UID in the selected pool; an empty history yields zero.
        pub async fn get_count_by_uid(uid: i32, pool: &PgPool) -> anyhow::Result<i64> {
            Ok(sqlx::query_file!($sql, uid)
                .fetch_one(pool)
                .await?
                .count
                .unwrap())
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
pub async fn set_all_by_pool(kind:crate::GachaType,set_all: &SetAll, connection: &mut sqlx::PgConnection)->anyhow::Result<u64> { match kind { $(crate::GachaType::$variant => $module::set_all(set_all, connection).await,)* } }
/// Routes a localized history read to its exact pool in ascending pull-ID order.
/// Invalid stored identity or missing rarity is an error; labels remain optional.
pub async fn get_by_uid_by_pool(kind:crate::GachaType,uid:i32, language:crate::Language, pool:&PgPool)->anyhow::Result<Vec<DbWarp>> { match kind { $(crate::GachaType::$variant => $module::get_by_uid(uid,language,pool).await,)* } }
/// Routes a minimum-timestamp lookup; returns None when this UID has no pulls in the pool.
pub async fn get_earliest_timestamp_by_uid_by_pool(kind:crate::GachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::GachaType::$variant => $module::get_earliest_timestamp_by_uid(uid,pool).await,)* } }
/// Routes a maximum-timestamp lookup; returns None when this UID has no pulls in the pool.
pub async fn get_latest_timestamp_by_uid_by_pool(kind:crate::GachaType,uid:i32,pool:&PgPool)->anyhow::Result<Option<DateTime<Utc>>> { match kind { $(crate::GachaType::$variant => $module::get_latest_timestamp_by_uid(uid,pool).await,)* } }
/// Routes a pull count to the exact pool, including empty histories and collab pools.
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

#[cfg(test)]
mod collection_count_tests {
    use super::*;

    #[sqlx::test]
    async fn sql_performance_collection_counts_preserve_copies_and_localization(pool: PgPool) {
        assert!(get_characters_count_by_uid(1, Language::En, &pool)
            .await
            .unwrap()
            .is_empty());
        assert!(get_light_cones_count_by_uid(1, Language::En, &pool)
            .await
            .unwrap()
            .is_empty());
        sqlx::query("INSERT INTO mihomo(uid, region, name, level, signature, avatar_icon, achievement_count)
            VALUES (1, 'na', 'seed', 1, '', '', 0), (2, 'eu', 'other', 1, '', '', 0)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO characters(id, rarity) VALUES (1, 5), (2, 4)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO light_cones(id, rarity) VALUES (3, 5), (4, 3)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO characters_text(id, language, name, element, path)
            SELECT id, lang, lang || id, 'element-' || lang, 'path-' || lang
            FROM characters CROSS JOIN UNNEST(ARRAY['en','fr']) lang",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO light_cones_text(id, language, name, path)
            SELECT id, lang, lang || id, 'path-' || lang
            FROM light_cones CROSS JOIN UNNEST(ARRAY['en','fr']) lang",
        )
        .execute(&pool)
        .await
        .unwrap();
        for table in [
            "warps_departure",
            "warps_standard",
            "warps_special",
            "warps_lc",
            "warps_collab",
            "warps_collab_lc",
        ] {
            // Every item appears repeatedly in every pool; another UID must not add copies.
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "INSERT INTO {table}(uid, id, character, light_cone, timestamp, official)
                 SELECT uid, i, CASE WHEN i % 4 < 2 THEN 1 + i % 4 END,
                    CASE WHEN i % 4 >= 2 THEN 1 + i % 4 END, now(), i % 2 = 0
                 FROM mihomo CROSS JOIN generate_series(1, 600) i"
            )))
            .execute(&pool)
            .await
            .unwrap();
        }
        let characters = get_characters_count_by_uid(1, Language::Fr, &pool)
            .await
            .unwrap();
        assert_eq!(
            characters
                .iter()
                .map(|c| (c.id, c.rarity, c.name.as_str(), c.count))
                .collect::<Vec<_>>(),
            vec![(1, 5, "fr1", Some(900)), (2, 4, "fr2", Some(900))]
        );
        assert!(characters.iter().all(|c| c.path == "path-fr"
            && c.path_id == "path-en"
            && c.element == "element-fr"
            && c.element_id == "element-en"));
        let cones = get_light_cones_count_by_uid(1, Language::Fr, &pool)
            .await
            .unwrap();
        assert_eq!(
            cones
                .iter()
                .map(|c| (c.id, c.rarity, c.name.as_str(), c.count))
                .collect::<Vec<_>>(),
            vec![(3, 5, "fr3", Some(900)), (4, 3, "fr4", Some(900))]
        );
        assert!(cones
            .iter()
            .all(|c| c.path == "path-fr" && c.path_id == "path-en"));
        // LEFT JOIN must retain counts with absent text; typed display reads still reject missing labels.
        sqlx::query("DELETE FROM characters_text WHERE id = 1")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM light_cones_text WHERE id = 3 AND language = 'fr'")
            .execute(&pool)
            .await
            .unwrap();
        for query in [
            include_str!("../../../sql/warps/get_characters_count_by_uid.sql"),
            include_str!("../../../sql/warps/get_light_cones_count_by_uid.sql"),
        ] {
            use sqlx::Row;
            let rows = sqlx::query(query)
                .bind(1_i32)
                .bind("fr")
                .fetch_all(&pool)
                .await
                .unwrap();
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].get::<i64, _>("count"), 900);
            assert_eq!(rows[0].get::<Option<String>, _>("name"), None);
        }
        assert!(get_characters_count_by_uid(1, Language::Fr, &pool)
            .await
            .is_err());
        assert!(get_light_cones_count_by_uid(1, Language::Fr, &pool)
            .await
            .is_err());
    }
}
