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

use crate::{api::ApiResult, database, GiGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/wishes-import")),
    paths(post_gi_wishes_import),
    components(schemas(WishesImportParams, WishesImport, WishesImportInfo, Status))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

lazy_static::lazy_static! {
    static ref DATA: web::Data<WishesImportInfos> = web::Data::new(WishesImportInfos::default());
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(DATA.clone())
        .service(post_gi_wishes_import)
        .configure(uid::configure);
}

#[derive(Deserialize)]
struct GachaLog {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    list: Vec<Entry>,
    region: String,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    uid: String,
    item_type: String,
    item_id: String,
    time: String,
}

type WishesImportInfos = Mutex<HashMap<i32, Arc<Mutex<WishesImportInfo>>>>;

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    Calculating,
    Finished,
    Error(String),
}

#[derive(Serialize, ToSchema, Clone)]
struct WishesImportInfo {
    gacha_type: GiGachaType,
    beginner: usize,
    standard: usize,
    character: usize,
    weapon: usize,
    chronicled: usize,
    status: Status,
}

#[derive(Deserialize, ToSchema)]
struct WishesImportParams {
    url: String,
}

#[derive(Serialize, ToSchema)]
struct WishesImport {
    uid: i32,
}

#[utoipa::path(
    tag = "gi/wishes-import",
    post,
    path = "/api/gi/wishes-import",
    request_body = WishesImportParams,
    responses(
        (status = 200, description = "WishesImport", body = WishesImport),
    )
)]
#[post("/api/gi/wishes-import")]
async fn post_gi_wishes_import(
    session: Session,
    params: web::Json<WishesImportParams>,
    wishes_import_infos: web::Data<WishesImportInfos>,
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
        Url::parse("https://public-operation-hk4e-sg.hoyoverse.com/gacha_info/api/getGachaLog")?;

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[("lang", "en"), ("size", "20")])
        .finish();

    let mut uid = 0;

    for gacha_type in [100, 200, 301, 302, 500] {
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
        let info = Arc::new(Mutex::new(WishesImportInfo {
            gacha_type: GiGachaType::Standard,
            beginner: 0,
            standard: 0,
            character: 0,
            weapon: 0,
            chronicled: 0,
            status: Status::Error("No data".to_string()),
        }));

        wishes_import_infos.lock().await.insert(uid, info.clone());

        return Ok(HttpResponse::Ok().json(WishesImport { uid }));
    }

    database::gi::uids::set(&database::gi::uids::DbUid { uid }, &pool).await?;
    if let Ok(Some(username)) = session.get::<String>("username") {
        let connection = database::gi::connections::DbConnection {
            uid,
            username,
            verified: true,
            private: false,
        };

        database::gi::connections::set(&connection, &pool).await?;
    }

    if wishes_import_infos.lock().await.contains_key(&uid) {
        return Ok(HttpResponse::Ok().json(WishesImport { uid }));
    }

    let info = Arc::new(Mutex::new(WishesImportInfo {
        gacha_type: GiGachaType::Standard,
        standard: 0,
        bangboo: 0,
        special: 0,
        w_engine: 0,
        status: Status::Pending,
    }));

    wishes_import_infos.lock().await.insert(uid, info.clone());

    rt::spawn(async move {
        let mut error = Ok(());

        for gacha_type in GiGachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_wishes(&url, gacha_type, &info, &pool).await {
                error = Err(e);

                break;
            }
        }

        if let Err(e) = error {
            info.lock().await.status = Status::Error(e.to_string());
        } else if let Err(e) = calculate_stats(uid, &info, &pool).await {
            info.lock().await.status = Status::Error(e.to_string());
        } else {
            info.lock().await.status = Status::Finished;
        }

        rt::spawn(async move {
            rt::time::sleep(Duration::from_secs(60)).await;

            wishes_import_infos.lock().await.remove(&uid);
        });
    });

    Ok(HttpResponse::Ok().json(WishesImport { uid }))
}

async fn import_wishes(
    url: &Url,
    gacha_type: GiGachaType,
    info: &Arc<Mutex<WishesImportInfo>>,
    pool: &PgPool,
) -> ApiResult<()> {
    let mut url = url.clone();
    let mut end_id = "0".to_string();

    url.query_pairs_mut()
        .extend_pairs(&[(
            "gacha_type",
            match gacha_type {
                GiGachaType::Standard => "1001",
                GiGachaType::Special => "2001",
                GiGachaType::WEngine => "3001",
                GiGachaType::Bangboo => "5001",
            },
        )])
        .finish();

    let mut set_all = database::gi::wishes::SetAll::default();

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
            let uid: i32 = entry.uid.parse()?;

            let exists = match gacha_type {
                GiGachaType::Standard => {
                    database::gi::wishes::standard::exists(id, uid, pool).await?
                }
                GiGachaType::Special => {
                    database::gi::wishes::special::exists(id, uid, pool).await?
                }
                GiGachaType::WEngine => {
                    database::gi::wishes::w_engine::exists(id, uid, pool).await?
                }
                GiGachaType::Bangboo => {
                    database::gi::wishes::bangboo::exists(id, uid, pool).await?
                }
            };

            if exists {
                continue;
            }

            let item: i32 = entry.item_id.parse()?;

            let mut character =
                (entry.item_type == "Agents" || entry.item_type == "代理人").then_some(item);
            let mut w_engine =
                (entry.item_type == "W-Engines" || entry.item_type == "音擎").then_some(item);
            let mut bangboo =
                (entry.item_type == "Bangboo" || entry.item_type == "邦布").then_some(item);

            if character.is_none() && w_engine.is_none() && bangboo.is_none() {
                if item >= 50000 {
                    bangboo = Some(item);
                } else if item >= 12000 {
                    w_engine = Some(item);
                } else {
                    character = Some(item);
                }
            }

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                - timestamp_offset;

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.w_engine.push(w_engine);
            set_all.bangboo.push(bangboo);
            set_all.timestamp.push(timestamp);
            set_all.official.push(true);

            match gacha_type {
                GiGachaType::Standard => info.lock().await.standard += 1,
                GiGachaType::Special => info.lock().await.special += 1,
                GiGachaType::WEngine => info.lock().await.w_engine += 1,
                GiGachaType::Bangboo => info.lock().await.bangboo += 1,
            }
        }
    }

    match gacha_type {
        GiGachaType::Standard => database::gi::wishes::standard::set_all(&set_all, pool).await?,
        GiGachaType::Special => database::gi::wishes::special::set_all(&set_all, pool).await?,
        GiGachaType::WEngine => database::gi::wishes::w_engine::set_all(&set_all, pool).await?,
        GiGachaType::Bangboo => database::gi::wishes::bangboo::set_all(&set_all, pool).await?,
    }

    Ok(())
}

async fn calculate_stats(
    uid: i32,
    info: &Arc<Mutex<WishesImportInfo>>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    info.lock().await.status = Status::Calculating;

    Ok(())
}
