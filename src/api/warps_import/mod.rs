mod uid;

use std::{collections::HashMap, sync::Arc, time::Duration};

use actix_session::Session;
use actix_web::{post, rt, web, HttpResponse, Responder};
use chrono::{FixedOffset, NaiveDateTime};
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
    region_time_zone: i32,
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
    Calculating,
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
    #[serde(default)]
    ignore_timestamps: bool,
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

    let mut url = Url::parse(
        "https://public-operation-hkrpg-sg.hoyoverse.com/common/gacha_record/api/getGachaLog",
    )?;

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[("lang", "en"), ("game_biz", "hkrpg_global"), ("size", "20")])
        .finish();

    let mut uid = None;

    for gacha_type in GachaType::iter().map(|gt| gt.id()) {
        let gacha_log: GachaLog = reqwest::get(format!("{url}&gacha_type={gacha_type}&end_id=0"))
            .await?
            .json()
            .await?;

        if let Some(entry) = gacha_log.data.list.first() {
            uid = Some(entry.uid.parse()?);
            break;
        }
    }

    let Some(uid) = uid else {
        let info = Arc::new(Mutex::new(WarpsImportInfo {
            gacha_type: GachaType::Standard,
            standard: 0,
            departure: 0,
            special: 0,
            lc: 0,
            status: Status::Error("No data".to_string()),
        }));

        warps_import_infos.lock().await.insert(0, info.clone());

        return Ok(HttpResponse::Ok().json(WarpsImport { uid: 0 }));
    };

    // Wacky way to update the database in case the uid isn't in there
    if !database::mihomo::exists(uid, &pool).await?
        && mihomo::get(uid, Language::En, &pool).await.is_err()
    {
        let region = match uid.to_string().chars().next() {
            Some('6') => "na",
            Some('7') => "eu",
            Some('8') | Some('9') => "asia",
            _ => "cn",
        }
        .to_string();

        let db_mihomo = database::mihomo::DbMihomo {
            uid,
            region,
            ..Default::default()
        };

        database::mihomo::set(&db_mihomo, &pool).await?;
    }

    if let Ok(Some(username)) = session.get::<String>("username") {
        let connection = database::connections::DbConnection {
            uid,
            username,
            verified: true,
            private: false,
        };

        database::connections::set(&connection, &pool).await?;
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
        let mut error = Ok(());

        for gacha_type in GachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_warps(
                uid,
                &url,
                params.ignore_timestamps,
                gacha_type,
                &info,
                &pool,
            )
            .await
            {
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

            warps_import_infos.lock().await.remove(&uid);
        });
    });

    Ok(HttpResponse::Ok().json(WarpsImport { uid }))
}

async fn import_warps(
    uid: i32,
    url: &Url,
    ignore_timestamps: bool,
    gacha_type: GachaType,
    info: &Arc<Mutex<WarpsImportInfo>>,
    pool: &PgPool,
) -> ApiResult<()> {
    let mut url = url.clone();
    let mut end_id = "0".to_string();

    url.query_pairs_mut()
        .extend_pairs(&[("gacha_type", &gacha_type.id().to_string())])
        .finish();

    let mut set_all = database::warps::SetAll::default();

    let latest_timestamp = match gacha_type {
        GachaType::Departure => {
            database::warps::departure::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GachaType::Standard => {
            database::warps::standard::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GachaType::Special => {
            database::warps::special::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GachaType::Lc => database::warps::lc::get_latest_timestamp_by_uid(uid, pool).await?,
    };

    'outer: loop {
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

        let tz = FixedOffset::east_opt(3600 * gacha_log.data.region_time_zone).unwrap();

        for entry in gacha_log.data.list {
            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?
                .and_local_timezone(tz)
                .unwrap()
                .to_utc();

            if !ignore_timestamps {
                if let Some(latest_timestamp) = latest_timestamp {
                    if timestamp <= latest_timestamp {
                        break 'outer;
                    }
                }
            }

            end_id.clone_from(&entry.id);

            let id = entry.id.parse()?;

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

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.light_cone.push(light_cone);
            set_all.timestamp.push(timestamp);
            set_all.official.push(true);

            match gacha_type {
                GachaType::Standard => info.lock().await.standard += 1,
                GachaType::Departure => info.lock().await.departure += 1,
                GachaType::Special => info.lock().await.special += 1,
                GachaType::Lc => info.lock().await.lc += 1,
            }
        }
    }

    match gacha_type {
        GachaType::Departure => database::warps::departure::set_all(&set_all, pool).await?,
        GachaType::Standard => database::warps::standard::set_all(&set_all, pool).await?,
        GachaType::Special => database::warps::special::set_all(&set_all, pool).await?,
        GachaType::Lc => database::warps::lc::set_all(&set_all, pool).await?,
    };

    Ok(())
}

async fn calculate_stats(
    uid: i32,
    info: &Arc<Mutex<WarpsImportInfo>>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    info.lock().await.status = Status::Calculating;

    info.lock().await.gacha_type = GachaType::Standard;
    calculate_stats_standard(uid, pool).await?;
    info.lock().await.gacha_type = GachaType::Special;
    calculate_stats_special(uid, pool).await?;
    info.lock().await.gacha_type = GachaType::Lc;
    calculate_stats_lc(uid, pool).await?;

    Ok(())
}

async fn calculate_stats_standard(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let warps = database::warps::standard::get_infos_by_uid(uid, pool).await?;

    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    for warp in &warps {
        pull_4 += 1;
        pull_5 += 1;

        match warp.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;
            }
            _ => {}
        }
    }

    let luck_4 = sum_4 as f64 / count_4 as f64;
    let luck_5 = sum_5 as f64 / count_5 as f64;

    let stat = database::warps_stats::standard::DbWarpsStatStandard {
        uid,
        luck_4,
        luck_5,
    };
    database::warps_stats::standard::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_special(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let mut banners: HashMap<_, Vec<_>> = HashMap::new();

    for banner in database::banners::get_all(&pool).await? {
        if let Some(character) = banner.character {
            banners
                .entry(character)
                .or_default()
                .push(banner.start..banner.end);
        }

        if let Some(light_cone) = banner.light_cone {
            banners
                .entry(light_cone)
                .or_default()
                .push(banner.start..banner.end);
        }
    }

    let warps = database::warps::special::get_infos_by_uid(uid, pool).await?;

    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for warp in &warps {
        pull_4 += 1;
        pull_5 += 1;

        match warp.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if banners
                        .get(&warp.character.unwrap())
                        .map(|v| v.iter().any(|r| r.contains(&warp.timestamp)))
                        .unwrap_or_default()
                    {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);

                        continue;
                    }

                    win_streak = 0;

                    loss_streak += 1;
                    max_loss_streak = max_loss_streak.max(loss_streak);

                    guarantee = true;
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = sum_4 as f64 / count_4 as f64;
    let luck_5 = sum_5 as f64 / count_5 as f64;
    let win_rate = sum_win as f64 / count_win as f64;

    let stat = database::warps_stats::special::DbWarpsStatSpecial {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::special::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_lc(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let mut banners: HashMap<_, Vec<_>> = HashMap::new();

    for banner in database::banners::get_all(&pool).await? {
        if let Some(character) = banner.character {
            banners
                .entry(character)
                .or_default()
                .push(banner.start..banner.end);
        }

        if let Some(light_cone) = banner.light_cone {
            banners
                .entry(light_cone)
                .or_default()
                .push(banner.start..banner.end);
        }
    }

    let warps = database::warps::lc::get_infos_by_uid(uid, pool).await?;

    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for warp in &warps {
        pull_4 += 1;
        pull_5 += 1;

        match warp.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if banners
                        .get(&warp.light_cone.unwrap())
                        .map(|v| v.iter().any(|r| r.contains(&warp.timestamp)))
                        .unwrap_or_default()
                    {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);

                        continue;
                    }

                    win_streak = 0;

                    loss_streak += 1;
                    max_loss_streak = max_loss_streak.max(loss_streak);

                    guarantee = true;
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = sum_4 as f64 / count_4 as f64;
    let luck_5 = sum_5 as f64 / count_5 as f64;
    let win_rate = sum_win as f64 / count_win as f64;

    let stat = database::warps_stats::lc::DbWarpsStatLc {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::lc::set(&stat, pool).await?;

    Ok(())
}
