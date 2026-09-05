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
    database, ZzzGachaType,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/signals-import")),
    paths(post_zzz_signals_import),
    components(schemas(SignalsImportParams, SignalsImport, SignalsImportInfo, ImportStatus, ImportErrorCode))
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

type SignalsImportInfos = ImportJobStore<SignalsImportInfo>;

#[derive(Serialize, ToSchema, Clone)]
struct SignalsImportInfo {
    gacha_type: ZzzGachaType,
    standard: usize,
    special: usize,
    w_engine: usize,
    bangboo: usize,
    exclusive_rescreening: usize,
    w_engine_reverberation: usize,
    status: ImportStatus,
}

#[derive(Deserialize, ToSchema)]
struct SignalsImportParams {
    url: String,
}

#[derive(Serialize, ToSchema)]
struct SignalsImport {
    uid: i32,
    /// Opaque identifier used to poll this import without exposing the UID-keyed job store.
    #[schema(value_type = String)]
    job_id: ImportJobId,
}

#[utoipa::path(
    tag = "zzz/signals-import",
    post,
    path = "/api/zzz/signals-import",
    request_body = SignalsImportParams,
    responses(
        (status = 200, description = "SignalsImport", body = SignalsImport),
        (status = 400, description = "Invalid URL"),
    )
)]
#[post("/api/zzz/signals-import")]
async fn post_zzz_signals_import(
    session: Session,
    params: web::Json<SignalsImportParams>,
    signals_import_infos: web::Data<SignalsImportInfos>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let url = match validate_import_url(&params.url) {
        Ok(url) => url,
        Err(response) => return Ok(response),
    };

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
    let mut import_error = None;

    for gacha_type in ZzzGachaType::iter().map(|gt| gt.id()) {
        let response =
            match reqwest::get(format!("{url}&real_gacha_type={gacha_type}&end_id=0")).await {
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
            uid = entry.uid.parse()?;
            break;
        }
    }

    if uid == 0 {
        let job = signals_import_infos
            .create(SignalsImportInfo {
                gacha_type: ZzzGachaType::Standard,
                standard: 0,
                bangboo: 0,
                special: 0,
                w_engine: 0,
                exclusive_rescreening: 0,
                w_engine_reverberation: 0,
                status: ImportStatus::Error(
                    import_error.unwrap_or(ImportErrorCode::InvalidResponse),
                ),
            })
            .await;
        let job_id = job.id;
        signals_import_infos.complete(job_id).await;

        return Ok(HttpResponse::Ok().json(SignalsImport { uid, job_id }));
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

    let job = signals_import_infos
        .start(
            uid,
            SignalsImportInfo {
                gacha_type: ZzzGachaType::Standard,
                standard: 0,
                bangboo: 0,
                special: 0,
                w_engine: 0,
                exclusive_rescreening: 0,
                w_engine_reverberation: 0,
                status: ImportStatus::Pending,
            },
        )
        .await;
    let job_id = job.id;

    if !job.is_new {
        return Ok(HttpResponse::Ok().json(SignalsImport { uid, job_id }));
    }

    let info = job.info;

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
            let code = classify_import_error(e.as_ref(), ImportErrorCode::PersistenceFailed);
            let gacha_type = info.lock().await.gacha_type;
            error!(game = "zzz", uid, pool = %gacha_type, %job_id, error = %redacted_error(e), "gacha import failed");
            info.lock().await.status = ImportStatus::Error(code);
        } else {
            info.lock().await.status = ImportStatus::Finished;
        }

        signals_import_infos.complete(job_id).await;
    });

    Ok(HttpResponse::Ok().json(SignalsImport { uid, job_id }))
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
        .extend_pairs(&[("real_gacha_type", gacha_type.id().to_string())])
        .finish();

    let mut pulls = Vec::new();

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

        let tz = (gacha_log.data.region_time_zone as i32)
            .checked_mul(3600)
            .and_then(FixedOffset::east_opt)
            .ok_or_else(|| anyhow::anyhow!("invalid timezone"))?;

        for entry in gacha_log.data.list {
            end_id.clone_from(&entry.id);

            let id = entry.id.parse()?;
            let uid: i32 = entry.uid.parse()?;

            let item: i32 = entry.item_id.parse()?;

            let item = match entry.item_type.as_str() {
                "Agents" | "代理人" => PullItem::Character(item),
                "W-Engines" | "音擎" => PullItem::WEngine(item),
                "Bangboo" | "邦布" => PullItem::Bangboo(item),
                _ if item >= 50000 => PullItem::Bangboo(item),
                _ if item >= 12000 => PullItem::WEngine(item),
                _ => PullItem::Character(item),
            };
            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?
                .and_local_timezone(tz)
                .single()
                .ok_or_else(|| anyhow::anyhow!("invalid local time"))?
                .to_utc();

            pulls.push(NormalizedPull {
                uid,
                id,
                pool: PullPool::Zzz(gacha_type),
                item,
                timestamp,
                provenance: PullProvenance::Official,
            });

            match gacha_type {
                ZzzGachaType::Standard => info.lock().await.standard += 1,
                ZzzGachaType::Special => info.lock().await.special += 1,
                ZzzGachaType::WEngine => info.lock().await.w_engine += 1,
                ZzzGachaType::Bangboo => info.lock().await.bangboo += 1,
                ZzzGachaType::ExclusiveRescreening => info.lock().await.exclusive_rescreening += 1,
                ZzzGachaType::WEngineReverberation => info.lock().await.w_engine_reverberation += 1,
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
