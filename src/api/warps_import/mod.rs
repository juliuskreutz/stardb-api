use crate::gacha::imports::{NormalizedPull, PullItem, PullPool, PullProvenance};
mod uid;

use std::{sync::Arc, time::Duration};

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
        import_jobs::{
            classify_import_error, redacted_error, ImportErrorCode, ImportJobId, ImportJobStore,
            ImportStatus,
        },
        validate_import_url, ApiResult,
    },
    database, mihomo, GachaType,
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
        warps_import_infos.complete(job_id).await;

        return Ok(HttpResponse::Ok().json(WarpsImport { uid: 0, job_id }));
    };

    mihomo::ensure_row(uid, &pool).await?;

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
        } else {
            info.lock().await.status = ImportStatus::Finished;
        }

        warps_import_infos.complete(job_id).await;
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

    let mut pulls = Vec::new();

    let latest_timestamp =
        database::warps::get_latest_timestamp_by_uid_by_pool(gacha_type, uid, pool).await?;

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

        let tz = (gacha_log.data.region_time_zone as i32)
            .checked_mul(3600)
            .and_then(FixedOffset::east_opt)
            .ok_or_else(|| anyhow::anyhow!("invalid timezone"))?;

        for entry in gacha_log.data.list {
            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?
                .and_local_timezone(tz)
                .single()
                .ok_or_else(|| anyhow::anyhow!("invalid local time"))?
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

            let item = match entry.item_type.as_str() {
                "Character" | "角色" => PullItem::Character(item),
                "Light Cone" | "光錐" => PullItem::LightCone(item),
                _ if item >= 20000 => PullItem::LightCone(item),
                _ if item <= 10000 => PullItem::Character(item),
                _ => return Err(anyhow::anyhow!("invalid item type").into()),
            };
            pulls.push(NormalizedPull {
                uid,
                id,
                pool: PullPool::Hsr(gacha_type),
                item,
                timestamp,
                provenance: PullProvenance::Official,
            });

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

    let batch = crate::gacha::imports::ImportBatch::new(
        pulls,
        crate::gacha::imports::PullProvenance::Official,
    )?;
    crate::gacha::imports::persist_batch_in_transaction(&batch, pool).await?;

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
