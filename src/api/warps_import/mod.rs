mod uid;

use std::{collections::HashMap, sync::Arc, time::Duration};

use actix_session::Session;
use actix_web::{post, rt, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::IntoEnumIterator;
use url::Url;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database, mihomo, GachaType, Language};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps-import")),
    paths(post_warps_import),
    components(schemas(WarpsImportParams, WarpsImport, WarpsImportInfo, Status))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

lazy_static::lazy_static! {
    static ref DATA: web::Data<WarpsImportInfos> = web::Data::new(WarpsImportInfos::default());
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(DATA.clone())
        .service(post_warps_import)
        .configure(uid::configure);
}

#[derive(Deserialize)]
struct GachaLog {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    list: Vec<Entry>,
    region_time_zone: i64,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    uid: String,
    item_type: String,
    item_id: String,
    time: String,
}

type WarpsImportInfos = Mutex<HashMap<i32, Arc<Mutex<WarpsImportInfo>>>>;

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    Finished,
    Error(String),
}

#[derive(Serialize, ToSchema, Clone)]
struct WarpsImportInfo {
    gacha_type: GachaType,
    standard: usize,
    departure: usize,
    special: usize,
    lc: usize,
    status: Status,
}

#[derive(Deserialize, ToSchema)]
struct WarpsImportParams {
    url: String,
}

#[derive(Serialize, ToSchema)]
struct WarpsImport {
    uid: i32,
}

#[utoipa::path(
    tag = "warps-import",
    post,
    path = "/api/warps-import",
    request_body = WarpsImportParams,
    responses(
        (status = 200, description = "WarpsImport", body = WarpsImport),
    )
)]
#[post("/api/warps-import")]
async fn post_warps_import(
    session: Session,
    params: web::Json<WarpsImportParams>,
    warps_import_infos: web::Data<WarpsImportInfos>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let url = Url::parse(&params.url)?;

    let query = url.query_pairs().filter(|(name, _)| {
        matches!(
            name.to_string().as_str(),
            "authkey" | "authkey_ver" | "sign_type"
        )
    });

    let mut url =
        Url::parse("https://api-os-takumi.mihoyo.com/common/gacha_record/api/getGachaLog")?;

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[("lang", "en"), ("game_biz", "hkrpg_global"), ("size", "20")])
        .finish();

    let mut uid = 0;

    for gacha_type in [1, 2, 11, 12] {
        let gacha_log: GachaLog = reqwest::get(format!("{url}&gacha_type={gacha_type}&end_id=0"))
            .await?
            .json()
            .await?;

        if let Some(entry) = gacha_log.data.list.first() {
            uid = entry.uid.parse()?;
            break;
        }
    }

    if uid == 0 {
        let info = Arc::new(Mutex::new(WarpsImportInfo {
            gacha_type: GachaType::Standard,
            standard: 0,
            departure: 0,
            special: 0,
            lc: 0,
            status: Status::Error("No data".to_string()),
        }));

        warps_import_infos.lock().await.insert(uid, info.clone());

        return Ok(HttpResponse::Ok().json(WarpsImport { uid }));
    }

    // Wacky way to update the database in case the uid isn't in there
    if !database::mihomo::exists(uid, &pool).await?
        && mihomo::get(uid, Language::En, &pool).await.is_err()
    {
        let db_mihomo = database::mihomo::DbMihomo {
            uid,
            ..Default::default()
        };

        database::mihomo::set(&db_mihomo, &pool).await?;
    }

    if let Ok(Some(username)) = session.get::<String>("username") {
        let connection = database::DbConnection {
            uid,
            username,
            verified: true,
            private: false,
        };

        database::set_connection(&connection, &pool).await?;
    }

    if warps_import_infos.lock().await.contains_key(&uid) {
        return Ok(HttpResponse::Ok().json(WarpsImport { uid }));
    }

    let info = Arc::new(Mutex::new(WarpsImportInfo {
        gacha_type: GachaType::Standard,
        standard: 0,
        departure: 0,
        special: 0,
        lc: 0,
        status: Status::Pending,
    }));

    warps_import_infos.lock().await.insert(uid, info.clone());

    rt::spawn(async move {
        let mut error = None;

        for gacha_type in GachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_warps(&url, gacha_type, &info, &pool).await {
                error = Some(e.to_string());

                break;
            }
        }

        if let Some(e) = error {
            info.lock().await.status = Status::Error(e);
        } else {
            info.lock().await.status = Status::Finished;
        }

        rt::spawn(async move {
            rt::time::sleep(Duration::from_secs(60)).await;

            warps_import_infos.lock().await.remove(&uid);
        });
    });

    Ok(HttpResponse::Ok().json(WarpsImport { uid }))
}

async fn import_warps(
    url: &Url,
    gacha_type: GachaType,
    info: &Arc<Mutex<WarpsImportInfo>>,
    pool: &PgPool,
) -> ApiResult<()> {
    let mut url = url.clone();
    let mut end_id = "0".to_string();

    url.query_pairs_mut()
        .extend_pairs(&[(
            "gacha_type",
            match gacha_type {
                GachaType::Standard => "1",
                GachaType::Departure => "2",
                GachaType::Special => "11",
                GachaType::Lc => "12",
            },
        )])
        .finish();

    let mut warp_id = Vec::new();
    let mut warp_uid = Vec::new();
    let mut warp_character = Vec::new();
    let mut warp_light_cone = Vec::new();
    let mut warp_gacha_type = Vec::new();
    let mut warp_timestamp = Vec::new();
    let mut warp_official = Vec::new();

    loop {
        let mut i = 0;
        let gacha_log = loop {
            let response = reqwest::get(format!("{url}&end_id={end_id}")).await?;
            if let Ok(gacha_log) = response.json::<GachaLog>().await {
                break gacha_log;
            }

            if i > 2 {
                return Err(anyhow::anyhow!("Unsure").into());
            }

            rt::time::sleep(Duration::from_secs(1)).await;

            i += 1;
        };

        if gacha_log.data.list.is_empty() {
            break;
        }

        let timestamp_offset = chrono::Duration::hours(gacha_log.data.region_time_zone);

        for entry in gacha_log.data.list {
            end_id.clone_from(&entry.id);

            let id = entry.id.parse()?;
            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                - timestamp_offset;

            // FIXME: Temp
            database::delete_warp_by_id_and_timestamp(
                id,
                timestamp + chrono::Duration::hours(1),
                pool,
            )
            .await?;
            database::delete_warp_by_id_and_timestamp(
                id,
                timestamp + chrono::Duration::hours(6),
                pool,
            )
            .await?;
            database::delete_warp_by_id_and_timestamp(
                id,
                timestamp + chrono::Duration::hours(13),
                pool,
            )
            .await?;
            // FIXME: Temp

            if database::get_warp_by_id_and_timestamp(id, timestamp, Language::En, pool)
                .await
                .is_ok()
            {
                continue;
            }

            let uid: i32 = entry.uid.parse()?;
            let item: i32 = entry.item_id.parse()?;

            let mut character =
                (entry.item_type == "Character" || entry.item_type == "角色").then_some(item);
            let mut light_cone =
                (entry.item_type == "Light Cone" || entry.item_type == "光錐").then_some(item);

            if character.is_none() && light_cone.is_none() {
                if item >= 20000 {
                    light_cone = Some(item);
                } else if item <= 10000 {
                    character = Some(item);
                } else {
                    return Err(anyhow::anyhow!("{} is weird...", entry.item_type).into());
                }
            }

            warp_id.push(id);
            warp_uid.push(uid);
            warp_character.push(character);
            warp_light_cone.push(light_cone);
            warp_gacha_type.push(gacha_type);
            warp_timestamp.push(timestamp);
            warp_official.push(true);

            match gacha_type {
                GachaType::Standard => info.lock().await.standard += 1,
                GachaType::Departure => info.lock().await.departure += 1,
                GachaType::Special => info.lock().await.special += 1,
                GachaType::Lc => info.lock().await.lc += 1,
            }
        }
    }

    database::set_all_warps(
        &warp_id,
        &warp_uid,
        &warp_gacha_type,
        &warp_character,
        &warp_light_cone,
        &warp_timestamp,
        &warp_official,
        pool,
    )
    .await?;

    Ok(())
}
