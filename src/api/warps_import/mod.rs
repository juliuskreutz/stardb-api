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

use crate::{
    api::{
        banner_helpers::{self, HSR_STANDARD},
        import_jobs::{
            classify_import_error, redacted_error, ImportErrorCode, ImportJobId, ImportJobStore,
            ImportStatus,
        },
        validate_import_url, ApiResult,
    },
    database,
    gacha::stats_math::average_or_zero,
    mihomo, GachaType, Language,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps-import")),
    paths(post_warps_import),
    components(schemas(WarpsImportParams, WarpsImport, WarpsImportInfo, ImportStatus, ImportErrorCode))
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

type WarpsImportInfos = ImportJobStore<WarpsImportInfo>;

#[derive(Serialize, ToSchema, Clone)]
struct WarpsImportInfo {
    gacha_type: GachaType,
    standard: usize,
    departure: usize,
    special: usize,
    lc: usize,
    collab: usize,
    collab_lc: usize,
    status: ImportStatus,
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
    /// Opaque identifier used to poll this import without exposing the UID-keyed job store.
    #[schema(value_type = String)]
    job_id: ImportJobId,
}

#[utoipa::path(
    tag = "warps-import",
    post,
    path = "/api/warps-import",
    request_body = WarpsImportParams,
    responses(
        (status = 200, description = "WarpsImport", body = WarpsImport),
        (status = 400, description = "Invalid URL"),
    )
)]
#[post("/api/warps-import")]
async fn post_warps_import(
    session: Session,
    params: web::Json<WarpsImportParams>,
    warps_import_infos: web::Data<WarpsImportInfos>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let original_url = match validate_import_url(&params.url) {
        Ok(url) => url,
        Err(response) => return Ok(response),
    };

    let mut uid = None;
    let mut import_error = None;

    // try to find the users uid. check each until we find one
    for gacha_type in GachaType::iter() {
        let gacha_type_id = gacha_type.id();
        let url = gacha_log_url(gacha_type, &original_url)?;

        let response =
            match reqwest::get(format!("{url}&gacha_type={gacha_type_id}&end_id=0")).await {
                Ok(response) => response,
                Err(error) => {
                    import_error = Some(classify_import_error(
                        &error,
                        ImportErrorCode::UpstreamUnavailable,
                    ));
                    continue;
                }
            };
        let gacha_log = match response.json::<GachaLog>().await {
            Ok(gacha_log) => gacha_log,
            Err(error) => {
                import_error = Some(classify_import_error(
                    &error,
                    ImportErrorCode::InvalidResponse,
                ));
                continue;
            }
        };

        if let Some(entry) = gacha_log.data.list.first() {
            uid = Some(entry.uid.parse()?);
            break;
        }
    }

    let Some(uid) = uid else {
        let job = warps_import_infos
            .create(WarpsImportInfo {
                gacha_type: GachaType::Standard,
                standard: 0,
                departure: 0,
                special: 0,
                lc: 0,
                collab: 0,
                collab_lc: 0,
                status: ImportStatus::Error(
                    import_error.unwrap_or(ImportErrorCode::InvalidResponse),
                ),
            })
            .await;
        let job_id = job.id;
        let jobs = warps_import_infos.clone();
        rt::spawn(async move {
            rt::time::sleep(Duration::from_secs(60)).await;
            jobs.remove(job_id).await;
        });

        return Ok(HttpResponse::Ok().json(WarpsImport { uid: 0, job_id }));
    };

    // Wacky way to update the database in case the uid isn't in there
    if !database::mihomo::exists(uid, &pool).await?
        && mihomo::get(uid, Language::En, &pool).await?.is_none()
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

    let job = warps_import_infos
        .start(
            uid,
            WarpsImportInfo {
                gacha_type: GachaType::Standard,
                standard: 0,
                departure: 0,
                special: 0,
                lc: 0,
                collab: 0,
                collab_lc: 0,
                status: ImportStatus::Pending,
            },
        )
        .await;
    let job_id = job.id;

    if !job.is_new {
        return Ok(HttpResponse::Ok().json(WarpsImport { uid, job_id }));
    }

    let info = job.info;

    rt::spawn(async move {
        let mut error = Ok(());

        for gacha_type in GachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_warps(
                uid,
                &original_url,
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
            let code = classify_import_error(e.as_ref(), ImportErrorCode::PersistenceFailed);
            let gacha_type = info.lock().await.gacha_type;
            error!(game = "hsr", uid, pool = %gacha_type, %job_id, error = %redacted_error(e), "gacha import failed");
            info.lock().await.status = ImportStatus::Error(code);
        } else if let Err(e) = calculate_stats(uid, &info, &pool).await {
            let gacha_type = info.lock().await.gacha_type;
            error!(game = "hsr", uid, pool = %gacha_type, %job_id, error = %redacted_error(e.into()), "gacha stats calculation failed");
            info.lock().await.status = ImportStatus::Error(ImportErrorCode::CalculationFailed);
        } else {
            info.lock().await.status = ImportStatus::Finished;
        }

        rt::spawn(async move {
            rt::time::sleep(Duration::from_secs(60)).await;

            warps_import_infos.remove(job_id).await;
        });
    });

    Ok(HttpResponse::Ok().json(WarpsImport { uid, job_id }))
}

async fn import_warps(
    uid: i32,
    original_url: &Url,
    ignore_timestamps: bool,
    gacha_type: GachaType,
    info: &Arc<Mutex<WarpsImportInfo>>,
    pool: &PgPool,
) -> ApiResult<()> {
    let mut url = gacha_log_url(gacha_type, original_url)?;
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
        GachaType::Collab => {
            database::warps::collab::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GachaType::CollabLc => {
            database::warps::collab_lc::get_latest_timestamp_by_uid(uid, pool).await?
        }
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
                GachaType::Collab => info.lock().await.collab += 1,
                GachaType::CollabLc => info.lock().await.collab_lc += 1,
            }
        }
    }

    match gacha_type {
        GachaType::Departure => database::warps::departure::set_all(&set_all, pool).await?,
        GachaType::Standard => database::warps::standard::set_all(&set_all, pool).await?,
        GachaType::Special => database::warps::special::set_all(&set_all, pool).await?,
        GachaType::Lc => database::warps::lc::set_all(&set_all, pool).await?,
        GachaType::Collab => database::warps::collab::set_all(&set_all, pool).await?,
        GachaType::CollabLc => database::warps::collab_lc::set_all(&set_all, pool).await?,
    };

    Ok(())
}

async fn calculate_stats(
    uid: i32,
    info: &Arc<Mutex<WarpsImportInfo>>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    info.lock().await.status = ImportStatus::Calculating;

    info.lock().await.gacha_type = GachaType::Standard;
    calculate_stats_standard(uid, pool).await?;
    info.lock().await.gacha_type = GachaType::Special;
    calculate_stats_special(uid, pool).await?;
    info.lock().await.gacha_type = GachaType::Lc;
    calculate_stats_lc(uid, pool).await?;
    info.lock().await.gacha_type = GachaType::Collab;
    calculate_stats_collab(uid, pool).await?;
    info.lock().await.gacha_type = GachaType::CollabLc;
    calculate_stats_collab_lc(uid, pool).await?;

    Ok(())
}

fn gacha_log_url(gacha_type: GachaType, original_url: &Url) -> Result<Url, url::ParseError> {
    let endpoint = gacha_log_endpoint(gacha_type);
    let query = original_url.query_pairs().filter(|(name, _)| {
        matches!(
            name.to_string().as_str(),
            "authkey" | "authkey_ver" | "sign_type"
        )
    });

    let api_name = if original_url.path().contains("/hkrpg_gacha_record/") {
        "hkrpg_gacha_record"
    } else {
        "gacha_record"
    };

    let mut url = Url::parse(&format!(
        "https://public-operation-hkrpg-sg.hoyoverse.com/common/{api_name}/api/{endpoint}",
    ))?;

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[("lang", "en"), ("game_biz", "hkrpg_global"), ("size", "20")])
        .finish();

    Ok(url)
}

fn gacha_log_endpoint(gacha_type: GachaType) -> &'static str {
    match gacha_type {
        // Collab banners have a different endpoint
        GachaType::Collab | GachaType::CollabLc => "getLdGachaLog",
        _ => "getGachaLog",
    }
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

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        // these don't apply to standard warps
        win_rate: 0.0,
        win_streak: 0,
        loss_streak: 0,
    };
    database::warps_stats::standard::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_special(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let mut banners: HashMap<_, Vec<_>> = HashMap::new();

    for banner in database::banners::get_all(pool).await? {
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

    let is_win = banner_helpers::is_win_fn(&banners, HSR_STANDARD);

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

                    if is_win(warp.character.unwrap(), warp.timestamp) {
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

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
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

    for banner in database::banners::get_all(pool).await? {
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

    let is_win = banner_helpers::is_win_fn(&banners, HSR_STANDARD);

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

                    if is_win(warp.light_cone.unwrap(), warp.timestamp) {
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

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
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

async fn calculate_stats_collab(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let mut banners: HashMap<_, Vec<_>> = HashMap::new();

    for banner in database::banners::get_all(pool).await? {
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

    let is_win = banner_helpers::is_win_fn(&banners, HSR_STANDARD);

    let warps = database::warps::collab::get_infos_by_uid(uid, pool).await?;

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

                    if is_win(warp.character.unwrap(), warp.timestamp) {
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

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::collab::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_collab_lc(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let mut banners: HashMap<_, Vec<_>> = HashMap::new();

    for banner in database::banners::get_all(pool).await? {
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

    let is_win = banner_helpers::is_win_fn(&banners, HSR_STANDARD);

    let warps = database::warps::collab_lc::get_infos_by_uid(uid, pool).await?;

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

                    if is_win(warp.light_cone.unwrap(), warp.timestamp) {
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

    let luck_4 = average_or_zero(sum_4, count_4);
    let luck_5 = average_or_zero(sum_5, count_5);
    let win_rate = average_or_zero(sum_win, count_win);

    let stat = database::warps_stats::DbWarpsStat {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::collab_lc::set(&stat, pool).await?;

    Ok(())
}
