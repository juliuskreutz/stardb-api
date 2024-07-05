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

use crate::{api::ApiResult, database, Language, ZzzGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/signals-import")),
    paths(post_zzz_signals_import),
    components(schemas(SignalsImportParams, SignalsImport, SignalsImportInfo, Status))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

lazy_static::lazy_static! {
    static ref DATA: web::Data<SignalsImportInfos> = web::Data::new(SignalsImportInfos::default());
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(DATA.clone())
        .service(post_zzz_signals_import)
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

type SignalsImportInfos = Mutex<HashMap<i32, Arc<Mutex<SignalsImportInfo>>>>;

#[derive(Serialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    Finished,
    Error(String),
}

#[derive(Serialize, ToSchema, Clone)]
struct SignalsImportInfo {
    gacha_type: ZzzGachaType,
    standard: usize,
    special: usize,
    w_engine: usize,
    bangboo: usize,
    status: Status,
}

#[derive(Deserialize, ToSchema)]
struct SignalsImportParams {
    url: String,
}

#[derive(Serialize, ToSchema)]
struct SignalsImport {
    uid: i32,
}

#[utoipa::path(
    tag = "zzz/signals-import",
    post,
    path = "/api/zzz/signals-import",
    request_body = SignalsImportParams,
    responses(
        (status = 200, description = "SignalsImport", body = SignalsImport),
    )
)]
#[post("/api/zzz/signals-import")]
async fn post_zzz_signals_import(
    session: Session,
    params: web::Json<SignalsImportParams>,
    signals_import_infos: web::Data<SignalsImportInfos>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let url = Url::parse(&params.url)?;

    let query = url.query_pairs().filter(|(name, _)| {
        matches!(
            name.to_string().as_str(),
            "authkey" | "authkey_ver" | "sign_type"
        )
    });

    let mut url = Url::parse(
        "https://public-operation-nap-sg.hoyoverse.com/common/gacha_record/api/getGachaLog",
    )?;

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[("lang", "en"), ("game_biz", "nap_global"), ("size", "20")])
        .finish();

    let mut uid = 0;

    for gacha_type in [1001, 2001, 3001, 5001] {
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
        let info = Arc::new(Mutex::new(SignalsImportInfo {
            gacha_type: ZzzGachaType::Standard,
            standard: 0,
            bangboo: 0,
            special: 0,
            w_engine: 0,
            status: Status::Error("No data".to_string()),
        }));

        signals_import_infos.lock().await.insert(uid, info.clone());

        return Ok(HttpResponse::Ok().json(SignalsImport { uid }));
    }

    database::zzz::uids::set(&database::zzz::uids::DbUid { uid }, &pool).await?;
    if let Ok(Some(username)) = session.get::<String>("username") {
        let connection = database::zzz::connections::DbConnection {
            uid,
            username,
            verified: true,
            private: false,
        };

        database::zzz::connections::set(&connection, &pool).await?;
    }

    if signals_import_infos.lock().await.contains_key(&uid) {
        return Ok(HttpResponse::Ok().json(SignalsImport { uid }));
    }

    let info = Arc::new(Mutex::new(SignalsImportInfo {
        gacha_type: ZzzGachaType::Standard,
        standard: 0,
        bangboo: 0,
        special: 0,
        w_engine: 0,
        status: Status::Pending,
    }));

    signals_import_infos.lock().await.insert(uid, info.clone());

    rt::spawn(async move {
        let mut error = None;

        for gacha_type in ZzzGachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_signals(&url, gacha_type, &info, &pool).await {
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

            signals_import_infos.lock().await.remove(&uid);
        });
    });

    Ok(HttpResponse::Ok().json(SignalsImport { uid }))
}

async fn import_signals(
    url: &Url,
    gacha_type: ZzzGachaType,
    info: &Arc<Mutex<SignalsImportInfo>>,
    pool: &PgPool,
) -> ApiResult<()> {
    let mut url = url.clone();
    let mut end_id = "0".to_string();

    //TODO: Here
    url.query_pairs_mut()
        .extend_pairs(&[(
            "gacha_type",
            match gacha_type {
                ZzzGachaType::Standard => "1001",
                ZzzGachaType::Special => "2001",
                ZzzGachaType::WEngine => "3001",
                ZzzGachaType::Bangboo => "5001",
            },
        )])
        .finish();

    let mut signal_id = Vec::new();
    let mut signal_uid = Vec::new();
    let mut signal_character = Vec::new();
    let mut signal_w_engine = Vec::new();
    let mut signal_bangboo = Vec::new();
    let mut signal_gacha_type = Vec::new();
    let mut signal_timestamp = Vec::new();
    let mut signal_official = Vec::new();

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

            if database::zzz::signals::get_by_id_and_timestamp(id, timestamp, Language::En, pool)
                .await
                .is_ok()
            {
                continue;
            }

            let uid: i32 = entry.uid.parse()?;
            let item: i32 = entry.item_id.parse()?;

            let mut character =
                (entry.item_type == "Agents" || entry.item_type == "角色").then_some(item);
            let mut w_engine =
                (entry.item_type == "W-Engines" || entry.item_type == "光錐").then_some(item);
            let mut bangboo =
                (entry.item_type == "Bangboo" || entry.item_type == "光錐").then_some(item);

            if character.is_none() && w_engine.is_none() {
                if item >= 50000 {
                    bangboo = Some(item);
                } else if item >= 12000 {
                    w_engine = Some(item);
                } else {
                    character = Some(item);
                }
            }

            signal_id.push(id);
            signal_uid.push(uid);
            signal_character.push(character);
            signal_w_engine.push(w_engine);
            signal_bangboo.push(bangboo);
            signal_gacha_type.push(gacha_type);
            signal_timestamp.push(timestamp);
            signal_official.push(true);

            match gacha_type {
                ZzzGachaType::Standard => info.lock().await.standard += 1,
                ZzzGachaType::Special => info.lock().await.special += 1,
                ZzzGachaType::WEngine => info.lock().await.w_engine += 1,
                ZzzGachaType::Bangboo => info.lock().await.bangboo += 1,
            }
        }
    }

    database::zzz::signals::set_all(
        &signal_id,
        &signal_uid,
        &signal_gacha_type,
        &signal_character,
        &signal_w_engine,
        &signal_bangboo,
        &signal_timestamp,
        &signal_official,
        pool,
    )
    .await?;

    Ok(())
}
