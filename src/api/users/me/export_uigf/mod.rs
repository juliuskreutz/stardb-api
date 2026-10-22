//! UIGF v4.1 exports with game-specific item labels, pool IDs, and server-local timestamps.
//! Each game keeps its own conversion policy while registry dispatch enumerates its pools.

use crate::gacha::imports::PullItem as StoredItem;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{FixedOffset, Utc};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, GachaType, GiGachaType, Language, ZzzGachaType};
use strum::IntoEnumIterator;

#[derive(utoipa::OpenApi)]
#[openapi(
    tags((name = "users/me/export-uigf")),
    paths(get_export_uigf),
    components(schemas(
        UIGFExport,
        UIGFInfo,
        UIGFGameEntry,
        UIGFListItem,
    ))
)]
struct ApiDoc;

/// Returns the OpenAPI fragment for this module’s routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module’s HTTP routes with the application.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_export_uigf);
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UIGFExport {
    info: UIGFInfo,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hk4e: Vec<UIGFGameEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hkrpg: Vec<UIGFGameEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    nap: Vec<UIGFGameEntry>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UIGFInfo {
    export_timestamp: u64,
    export_app: String,
    export_app_version: String,
    version: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UIGFGameEntry {
    uid: i32,
    timezone: i32,
    lang: String,
    list: Vec<UIGFListItem>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UIGFListItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    uigf_gacha_type: Option<String>,
    gacha_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    gacha_id: Option<String>,
    item_id: String,
    count: String,
    time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rank_type: Option<String>,
    id: String,
}

/// Returns the game’s fixed server offset in hours inferred from the UID prefix.
/// Unknown prefixes retain the existing UTC+8 fallback; daylight saving is not applied.
fn get_timezone_offset(uid: i32, game: &str) -> i32 {
    let uid_str = uid.to_string();
    let first_char = uid_str.chars().next().unwrap_or('0');
    let first_two: String = uid_str.chars().take(2).collect();

    match game {
        "hsr" => match first_char {
            '6' => -5,
            '7' => 1,
            '8' | '9' => 8,
            _ => 8,
        },
        "zzz" => match first_two.as_str() {
            "10" => -5,
            "15" => 1,
            _ => 8,
        },
        "gi" => match first_char {
            '6' => -5,
            '7' => 1,
            _ => 8,
        },
        _ => 8,
    }
}

/// Formats a UTC instant in the selected fixed server offset without a timezone suffix.
/// The offset must be valid for chrono; callers use the bounded server-offset mapping.
fn format_uigf_time(chrono_datetime: chrono::DateTime<Utc>, offset_hours: i32) -> String {
    let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap();
    let local_time = chrono_datetime.with_timezone(&offset);
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Converts a validated HSR row using UIGF’s exact Character/Light Cone labels.
/// Keeps nullable names, string IDs, and the caller’s pool ID and server offset.
fn warp_to_uigf_item(
    warp: database::warps::DbWarp,
    gacha_type: &str,
    gacha_id: Option<&str>,
    offset: i32,
) -> UIGFListItem {
    let item_id = warp.item.id().to_string();
    let item_type = Some(
        match warp.item {
            StoredItem::Character(_) => "Character",
            StoredItem::LightCone(_) => "Light Cone",
            _ => unreachable!("read model validates game item kinds"),
        }
        .to_string(),
    );

    UIGFListItem {
        uigf_gacha_type: None,
        gacha_type: gacha_type.to_string(),
        gacha_id: gacha_id.map(|s| s.to_string()),
        item_id,
        count: "1".to_string(),
        time: format_uigf_time(warp.timestamp, offset),
        name: warp.name,
        item_type,
        rank_type: Some(warp.rarity.to_string()),
        id: warp.id.to_string(),
    }
}

/// Converts a validated ZZZ row using UIGF’s exact Agents/W-Engines/Bangboo labels.
/// Keeps nullable names and emits the caller’s current pool ID, not a legacy alias.
fn signal_to_uigf_item(
    signal: database::zzz::signals::DbSignal,
    gacha_type: &str,
    offset: i32,
) -> UIGFListItem {
    let item_id = signal.item.id().to_string();
    let item_type = Some(
        match signal.item {
            StoredItem::Character(_) => "Agents",
            StoredItem::WEngine(_) => "W-Engines",
            StoredItem::Bangboo(_) => "Bangboo",
            _ => unreachable!("read model validates game item kinds"),
        }
        .to_string(),
    );

    UIGFListItem {
        uigf_gacha_type: None,
        gacha_type: gacha_type.to_string(),
        gacha_id: None,
        item_id,
        count: "1".to_string(),
        time: format_uigf_time(signal.timestamp, offset),
        name: signal.name,
        item_type,
        rank_type: Some(signal.rarity.to_string()),
        id: signal.id.to_string(),
    }
}

/// Converts a validated GI row using UIGF’s Character/Weapon labels.
/// Keeps both gacha-type fields explicit and formats time in the caller’s server offset.
fn wish_to_uigf_item(
    wish: database::gi::wishes::DbWish,
    gacha_type: &str,
    uigf_gacha_type: &str,
    offset: i32,
) -> UIGFListItem {
    let item_id = wish.item.id().to_string();
    let item_type = Some(
        match wish.item {
            StoredItem::Character(_) => "Character",
            StoredItem::Weapon(_) => "Weapon",
            _ => unreachable!("read model validates game item kinds"),
        }
        .to_string(),
    );

    UIGFListItem {
        uigf_gacha_type: Some(uigf_gacha_type.to_string()),
        gacha_type: gacha_type.to_string(),
        gacha_id: None,
        item_id,
        count: "1".to_string(),
        time: format_uigf_time(wish.timestamp, offset),
        name: wish.name,
        item_type,
        rank_type: Some(wish.rarity.to_string()),
        id: wish.id.to_string(),
    }
}

#[utoipa::path(
    tag = "users/me/export-uigf",
    get,
    path = "/api/users/me/export-uigf",
    responses(
        (status = 200, description = "UIGF Export", body = UIGFExport),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/export-uigf")]
/// Exports all connected histories in UIGF v4.1; requires a session username.
/// Sorts each UID’s list by formatted time then string ID, preserving insertion
/// order for ties, and omits empty UID lists. Read/conversion errors abort the export.
async fn get_export_uigf(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    // HSR (hkrpg)
    let mut hkrpg = Vec::new();
    for connection in database::connections::get_by_username(&username, &pool).await? {
        let uid = connection.uid;
        let offset = get_timezone_offset(uid, "hsr");

        let mut list = Vec::new();

        for kind in hsr_export_pools() {
            let id = kind.id().to_string();
            for warp in database::warps::get_by_uid_by_pool(kind, uid, Language::En, &pool).await? {
                list.push(warp_to_uigf_item(warp, &id, Some(&id), offset));
            }
        }

        list.sort_by(|a, b| a.time.cmp(&b.time).then(a.id.cmp(&b.id)));

        if !list.is_empty() {
            hkrpg.push(UIGFGameEntry {
                uid,
                timezone: offset,
                lang: "en-us".to_string(),
                list,
            });
        }
    }

    // ZZZ (nap)
    let mut nap = Vec::new();
    for connection in database::zzz::connections::get_by_username(&username, &pool).await? {
        let uid = connection.uid;
        let offset = get_timezone_offset(uid, "zzz");

        let mut list = Vec::new();

        for kind in ZzzGachaType::iter() {
            let id = kind.id().to_string();
            for signal in
                database::zzz::signals::get_by_uid_by_pool(kind, uid, Language::En, &pool).await?
            {
                list.push(signal_to_uigf_item(signal, &id, offset));
            }
        }

        list.sort_by(|a, b| a.time.cmp(&b.time).then(a.id.cmp(&b.id)));

        if !list.is_empty() {
            nap.push(UIGFGameEntry {
                uid,
                timezone: offset,
                lang: "en-us".to_string(),
                list,
            });
        }
    }

    // GI (hk4e)
    let mut hk4e = Vec::new();
    for connection in database::gi::connections::get_by_username(&username, &pool).await? {
        let uid = connection.uid;
        let offset = get_timezone_offset(uid, "gi");

        let mut list = Vec::new();

        for kind in GiGachaType::iter() {
            let id = gi_uigf_type(kind);
            for wish in
                database::gi::wishes::get_by_uid_by_pool(kind, uid, Language::En, &pool).await?
            {
                list.push(wish_to_uigf_item(wish, id, id, offset));
            }
        }

        list.sort_by(|a, b| a.time.cmp(&b.time).then(a.id.cmp(&b.id)));

        if !list.is_empty() {
            hk4e.push(UIGFGameEntry {
                uid,
                timezone: offset,
                lang: "en-us".to_string(),
                list,
            });
        }
    }

    let export = UIGFExport {
        info: UIGFInfo {
            export_timestamp: Utc::now().timestamp() as u64,
            export_app: "stardb".to_string(),
            export_app_version: "v0".to_string(),
            version: "v4.1".to_string(),
        },
        hk4e,
        hkrpg,
        nap,
    };

    Ok(HttpResponse::Ok().json(export))
}

#[cfg(test)]
mod serialization_tests {
    use super::*;
    #[test]
    fn every_item_kind_preserves_export_wire_strings() {
        let row = database::warps::DbWarp {
            id: 42,
            item: StoredItem::Character(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(warp_to_uigf_item(row, "11", Some("11"), 0)).unwrap();
        assert_eq!(value["item_type"], "Character");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
        let row = database::warps::DbWarp {
            id: 42,
            item: StoredItem::LightCone(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(warp_to_uigf_item(row, "11", Some("11"), 0)).unwrap();
        assert_eq!(value["item_type"], "Light Cone");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
        let row = database::gi::wishes::DbWish {
            id: 42,
            item: StoredItem::Character(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(wish_to_uigf_item(row, "301", "301", 0)).unwrap();
        assert_eq!(value["item_type"], "Character");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
        let row = database::gi::wishes::DbWish {
            id: 42,
            item: StoredItem::Weapon(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(wish_to_uigf_item(row, "301", "301", 0)).unwrap();
        assert_eq!(value["item_type"], "Weapon");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
        let row = database::zzz::signals::DbSignal {
            id: 42,
            item: StoredItem::Character(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(signal_to_uigf_item(row, "2", 0)).unwrap();
        assert_eq!(value["item_type"], "Agents");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
        let row = database::zzz::signals::DbSignal {
            id: 42,
            item: StoredItem::WEngine(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(signal_to_uigf_item(row, "2", 0)).unwrap();
        assert_eq!(value["item_type"], "W-Engines");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
        let row = database::zzz::signals::DbSignal {
            id: 42,
            item: StoredItem::Bangboo(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(signal_to_uigf_item(row, "2", 0)).unwrap();
        assert_eq!(value["item_type"], "Bangboo");
        assert_eq!(value["item_id"], "1700000000");
        assert_eq!(value["rank_type"], "4");
        assert_eq!(value["time"], "2023-11-14 22:13:20");
    }
}

/// Preserve the original insertion order for equal timestamp/id keys in stable sorting.
fn hsr_export_pools() -> impl Iterator<Item = GachaType> {
    std::iter::once(GachaType::Departure)
        .chain(GachaType::iter().filter(|kind| *kind != GachaType::Departure))
}

/// Returns the canonical UIGF pool ID for each Genshin pool; exports emit no legacy aliases.
fn gi_uigf_type(kind: GiGachaType) -> &'static str {
    match kind {
        GiGachaType::Beginner => "100",
        GiGachaType::Standard => "200",
        GiGachaType::Character => "301",
        GiGachaType::Weapon => "302",
        GiGachaType::Chronicled => "500",
    }
}

#[cfg(test)]
mod pool_order_tests {
    use super::*;
    #[test]
    fn all_pool_ids_and_stable_tie_order_match_existing_exports() {
        assert_eq!(
            hsr_export_pools().map(GachaType::id).collect::<Vec<_>>(),
            [2, 1, 11, 12, 21, 22]
        );
        assert_eq!(
            ZzzGachaType::iter()
                .map(ZzzGachaType::id)
                .collect::<Vec<_>>(),
            [1, 2, 3, 5, 102, 103]
        );
        assert_eq!(
            GiGachaType::iter().map(gi_uigf_type).collect::<Vec<_>>(),
            ["100", "200", "301", "302", "500"]
        );
    }
}
