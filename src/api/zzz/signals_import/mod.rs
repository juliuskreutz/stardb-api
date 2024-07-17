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

use crate::{api::ApiResult, database, ZzzGachaType};

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
    Calculating,
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
        let mut error = Ok(());

        for gacha_type in ZzzGachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_signals(&url, gacha_type, &info, &pool).await {
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

    let mut set_all = database::zzz::signals::SetAll::default();

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
                ZzzGachaType::Standard => {
                    database::zzz::signals::standard::exists(id, uid, pool).await?
                }
                ZzzGachaType::Special => {
                    database::zzz::signals::special::exists(id, uid, pool).await?
                }
                ZzzGachaType::WEngine => {
                    database::zzz::signals::w_engine::exists(id, uid, pool).await?
                }
                ZzzGachaType::Bangboo => {
                    database::zzz::signals::bangboo::exists(id, uid, pool).await?
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
                ZzzGachaType::Standard => info.lock().await.standard += 1,
                ZzzGachaType::Special => info.lock().await.special += 1,
                ZzzGachaType::WEngine => info.lock().await.w_engine += 1,
                ZzzGachaType::Bangboo => info.lock().await.bangboo += 1,
            }
        }
    }

    match gacha_type {
        ZzzGachaType::Standard => database::zzz::signals::standard::set_all(&set_all, pool).await?,
        ZzzGachaType::Special => database::zzz::signals::special::set_all(&set_all, pool).await?,
        ZzzGachaType::WEngine => database::zzz::signals::w_engine::set_all(&set_all, pool).await?,
        ZzzGachaType::Bangboo => database::zzz::signals::bangboo::set_all(&set_all, pool).await?,
    }

    Ok(())
}

async fn calculate_stats(
    uid: i32,
    info: &Arc<Mutex<SignalsImportInfo>>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    info.lock().await.status = Status::Calculating;

    info.lock().await.gacha_type = ZzzGachaType::Standard;
    calculate_stats_standard(uid, pool).await?;
    info.lock().await.gacha_type = ZzzGachaType::Special;
    calculate_stats_special(uid, pool).await?;
    info.lock().await.gacha_type = ZzzGachaType::WEngine;
    calculate_stats_w_engine(uid, pool).await?;
    info.lock().await.gacha_type = ZzzGachaType::Bangboo;
    calculate_stats_bangboo(uid, pool).await?;

    Ok(())
}

async fn calculate_stats_standard(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::standard::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;
            }
            _ => {}
        }
    }

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };

    let set_data = database::zzz::signals_stats::standard::SetData {
        uid,
        luck_a,
        luck_s,
    };
    database::zzz::signals_stats::standard::set_data(&set_data, pool).await?;

    Ok(())
}

async fn calculate_stats_special(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::special::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [1021, 1041, 1101, 1141, 1181, 1211].contains(&signal.character.unwrap()) {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    let set_data = database::zzz::signals_stats::special::SetData {
        uid,
        luck_a,
        luck_s,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::zzz::signals_stats::special::set_data(&set_data, pool).await?;

    Ok(())
}

async fn calculate_stats_w_engine(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::w_engine::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [14103, 14104, 14110, 14114, 14118, 14121]
                        .contains(&signal.w_engine.unwrap())
                    {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    let set_data = database::zzz::signals_stats::w_engine::SetData {
        uid,
        luck_a,
        luck_s,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::zzz::signals_stats::w_engine::set_data(&set_data, pool).await?;

    Ok(())
}

async fn calculate_stats_bangboo(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::bangboo::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;
            }
            _ => {}
        }
    }

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };

    let set_data = database::zzz::signals_stats::bangboo::SetData {
        uid,
        luck_a,
        luck_s,
    };
    database::zzz::signals_stats::bangboo::set_data(&set_data, pool).await?;

    Ok(())
}
