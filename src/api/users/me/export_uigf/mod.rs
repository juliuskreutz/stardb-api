use crate::gacha::imports::PullItem as StoredItem;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{FixedOffset, Utc};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, Language, ZzzGachaType};

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

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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

fn format_uigf_time(chrono_datetime: chrono::DateTime<Utc>, offset_hours: i32) -> String {
    let offset = FixedOffset::east_opt(offset_hours * 3600).unwrap();
    let local_time = chrono_datetime.with_timezone(&offset);
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

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

        for warp in database::warps::departure::get_by_uid(uid, Language::En, &pool).await? {
            list.push(warp_to_uigf_item(warp, "2", Some("2"), offset));
        }
        for warp in database::warps::standard::get_by_uid(uid, Language::En, &pool).await? {
            list.push(warp_to_uigf_item(warp, "1", Some("1"), offset));
        }
        for warp in database::warps::special::get_by_uid(uid, Language::En, &pool).await? {
            list.push(warp_to_uigf_item(warp, "11", Some("11"), offset));
        }
        for warp in database::warps::lc::get_by_uid(uid, Language::En, &pool).await? {
            list.push(warp_to_uigf_item(warp, "12", Some("12"), offset));
        }
        for warp in database::warps::collab::get_by_uid(uid, Language::En, &pool).await? {
            list.push(warp_to_uigf_item(warp, "21", Some("21"), offset));
        }
        for warp in database::warps::collab_lc::get_by_uid(uid, Language::En, &pool).await? {
            list.push(warp_to_uigf_item(warp, "22", Some("22"), offset));
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

        for signal in database::zzz::signals::standard::get_by_uid(uid, Language::En, &pool).await?
        {
            list.push(signal_to_uigf_item(
                signal,
                &ZzzGachaType::Standard.id().to_string(),
                offset,
            ));
        }
        for signal in database::zzz::signals::special::get_by_uid(uid, Language::En, &pool).await? {
            list.push(signal_to_uigf_item(
                signal,
                &ZzzGachaType::Special.id().to_string(),
                offset,
            ));
        }
        for signal in database::zzz::signals::w_engine::get_by_uid(uid, Language::En, &pool).await?
        {
            list.push(signal_to_uigf_item(
                signal,
                &ZzzGachaType::WEngine.id().to_string(),
                offset,
            ));
        }
        for signal in database::zzz::signals::bangboo::get_by_uid(uid, Language::En, &pool).await? {
            list.push(signal_to_uigf_item(
                signal,
                &ZzzGachaType::Bangboo.id().to_string(),
                offset,
            ));
        }
        for signal in
            database::zzz::signals::exclusive_rescreening::get_by_uid(uid, Language::En, &pool)
                .await?
        {
            list.push(signal_to_uigf_item(
                signal,
                &ZzzGachaType::ExclusiveRescreening.id().to_string(),
                offset,
            ));
        }
        for signal in
            database::zzz::signals::w_engine_reverberation::get_by_uid(uid, Language::En, &pool)
                .await?
        {
            list.push(signal_to_uigf_item(
                signal,
                &ZzzGachaType::WEngineReverberation.id().to_string(),
                offset,
            ));
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

        for wish in database::gi::wishes::beginner::get_by_uid(uid, Language::En, &pool).await? {
            list.push(wish_to_uigf_item(wish, "100", "100", offset));
        }
        for wish in database::gi::wishes::standard::get_by_uid(uid, Language::En, &pool).await? {
            list.push(wish_to_uigf_item(wish, "200", "200", offset));
        }
        for wish in database::gi::wishes::character::get_by_uid(uid, Language::En, &pool).await? {
            list.push(wish_to_uigf_item(wish, "301", "301", offset));
        }
        for wish in database::gi::wishes::weapon::get_by_uid(uid, Language::En, &pool).await? {
            list.push(wish_to_uigf_item(wish, "302", "302", offset));
        }
        for wish in database::gi::wishes::chronicled::get_by_uid(uid, Language::En, &pool).await? {
            list.push(wish_to_uigf_item(wish, "500", "500", offset));
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
