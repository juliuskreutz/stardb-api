mod uid;

use std::{collections::HashMap, sync::Arc, time::Duration};

use actix_web::{post, rt, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumIter, IntoEnumIterator};
use url::Url;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps-import")),
    paths(post_warps_import),
    components(schemas(WarpsImportParams, WarpsImport, WarpsImportInfo, GachaType, Status))
)]
struct ApiDoc;

#[derive(Display, EnumIter, Serialize, Deserialize, ToSchema, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum GachaType {
    Standard,
    Departure,
    Special,
    Lc,
}

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
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    uid: String,
    item_type: String,
    item_id: String,
    time: String,
}

type WarpsImportInfos = Mutex<HashMap<i64, Arc<Mutex<WarpsImportInfo>>>>;

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
    uid: i64,
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

    let gacha_log: GachaLog = reqwest::get(format!("{url}&gacha_type=11&end_id=0"))
        .await?
        .json()
        .await?;

    let uid = gacha_log.data.list[0].uid.parse()?;

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
    gt: GachaType,
    info: &Arc<Mutex<WarpsImportInfo>>,
    pool: &PgPool,
) -> ApiResult<()> {
    let mut url = url.clone();
    let mut end_id = "0".to_string();

    url.query_pairs_mut()
        .extend_pairs(&[(
            "gacha_type",
            match gt {
                GachaType::Standard => "1",
                GachaType::Departure => "2",
                GachaType::Special => "11",
                GachaType::Lc => "12",
            },
        )])
        .finish();

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

        for entry in gacha_log.data.list {
            end_id = entry.id.clone();

            let id = entry.id.parse()?;
            let gacha_type = gt.to_string();

            if database::get_warp_by_id_and_gacha_type(id, &gacha_type, "en", pool)
                .await
                .is_ok()
            {
                continue;
            }

            let uid = entry.uid.parse()?;
            let timestamp =
                NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?.and_utc();

            let item = entry.item_id.parse()?;

            {
                let gacha_type = gacha_type.clone();

                let character = (entry.item_type == "Character").then_some(item);
                let light_cone = (entry.item_type == "Light Cone").then_some(item);

                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character,
                    light_cone,
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, pool).await?;
            }

            match gt {
                GachaType::Standard => info.lock().await.standard += 1,
                GachaType::Departure => info.lock().await.departure += 1,
                GachaType::Special => info.lock().await.special += 1,
                GachaType::Lc => info.lock().await.lc += 1,
            }
        }
    }

    Ok(())
}
