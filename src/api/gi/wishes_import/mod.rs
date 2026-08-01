mod uid;

use std::{sync::Arc, time::Duration};

use actix_session::Session;
use actix_web::{post, rt, web, HttpResponse, Responder};
use chrono::{FixedOffset, NaiveDateTime};
use futures::lock::Mutex;
use reqwest::header;
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
    database, GiGachaType,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/wishes-import")),
    paths(post_gi_wishes_import),
    components(schemas(WishesImportParams, WishesImport, WishesImportInfo, ImportStatus, ImportErrorCode))
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
    name: String,
    time: String,
}

type WishesImportInfos = ImportJobStore<WishesImportInfo>;

#[derive(Serialize, ToSchema, Clone)]
struct WishesImportInfo {
    gacha_type: GiGachaType,
    beginner: usize,
    standard: usize,
    character: usize,
    weapon: usize,
    chronicled: usize,
    status: ImportStatus,
}

#[derive(Deserialize, ToSchema)]
struct WishesImportParams {
    url: String,
    #[serde(default)]
    ignore_timestamps: bool,
}

#[derive(Serialize, ToSchema)]
struct WishesImport {
    uid: i32,
    /// Opaque identifier used to poll this import without exposing the UID-keyed job store.
    #[schema(value_type = String)]
    job_id: ImportJobId,
}

#[utoipa::path(
    tag = "gi/wishes-import",
    post,
    path = "/api/gi/wishes-import",
    request_body = WishesImportParams,
    responses(
        (status = 200, description = "WishesImport", body = WishesImport),
        (status = 400, description = "Invalid URL"),
    )
)]
#[post("/api/gi/wishes-import")]
async fn post_gi_wishes_import(
    session: Session,
    params: web::Json<WishesImportParams>,
    wishes_import_infos: web::Data<WishesImportInfos>,
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

    let mut url = match url.domain() {
        Some("public-operation-hk4e.mihoyo.com") => {
            Url::parse("https://public-operation-hk4e.mihoyo.com/gacha_info/api/getGachaLog")?
        }
        _ => {
            Url::parse("https://public-operation-hk4e-sg.hoyoverse.com/gacha_info/api/getGachaLog")?
        }
    };

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[("lang", "en"), ("size", "20")])
        .finish();

    let mut uid = 0;
    let mut import_error = None;

    for gacha_type in [100, 200, 301, 302, 500] {
        // User-provided wish URLs can point at expired or malformed upstream responses.
        // Keep those as import status errors instead of bubbling reqwest decode errors.
        let gacha_log = match reqwest::get(format!("{url}&gacha_type={gacha_type}&end_id=0")).await
        {
            Ok(response) => match response.json::<GachaLog>().await {
                Ok(gacha_log) => gacha_log,
                Err(error) => {
                    import_error = Some(classify_import_error(
                        &error,
                        ImportErrorCode::InvalidResponse,
                    ));
                    continue;
                }
            },
            Err(error) => {
                import_error = Some(classify_import_error(
                    &error,
                    ImportErrorCode::UpstreamUnavailable,
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
        let job = wishes_import_infos
            .create(WishesImportInfo {
                gacha_type: GiGachaType::Standard,
                beginner: 0,
                standard: 0,
                character: 0,
                weapon: 0,
                chronicled: 0,
                status: ImportStatus::Error(
                    import_error.unwrap_or(ImportErrorCode::InvalidResponse),
                ),
            })
            .await;
        let job_id = job.id;
        let jobs = wishes_import_infos.clone();
        rt::spawn(async move {
            rt::time::sleep(Duration::from_secs(60)).await;
            jobs.remove(job_id).await;
        });

        return Ok(HttpResponse::Ok().json(WishesImport { uid, job_id }));
    }

    // Enka is only used to populate a display name. The import can continue without it.
    let name = match reqwest::Client::new()
        .get(format!("https://enka.network/api/uid/{uid}?info"))
        .header(header::USER_AGENT, "stardb")
        .send()
        .await
    {
        Ok(response) => match response.json::<serde_json::Value>().await {
            Ok(json) => json["playerInfo"]["nickname"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    };

    database::gi::profiles::set(&database::gi::profiles::DbProfile { uid, name }, &pool).await?;
    if let Ok(Some(username)) = session.get::<String>("username") {
        let connection = database::gi::connections::DbConnection {
            uid,
            username,
            verified: true,
            private: false,
        };

        database::gi::connections::set(&connection, &pool).await?;
    }

    let job = wishes_import_infos
        .start(
            uid,
            WishesImportInfo {
                gacha_type: GiGachaType::Standard,
                beginner: 0,
                standard: 0,
                character: 0,
                weapon: 0,
                chronicled: 0,
                status: ImportStatus::Pending,
            },
        )
        .await;
    let job_id = job.id;

    if !job.is_new {
        return Ok(HttpResponse::Ok().json(WishesImport { uid, job_id }));
    }

    let info = job.info;

    rt::spawn(async move {
        let mut error = Ok(());

        for gacha_type in GiGachaType::iter() {
            info.lock().await.gacha_type = gacha_type;

            if let Err(e) = import_wishes(
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
            let code = classify_import_error(e.as_ref(), ImportErrorCode::PersistenceFailed);
            let gacha_type = info.lock().await.gacha_type;
            error!(game = "gi", uid, pool = %gacha_type, %job_id, error = %redacted_error(e), "gacha import failed");
            info.lock().await.status = ImportStatus::Error(code);
        } else {
            info.lock().await.status = ImportStatus::Finished;
        }

        rt::spawn(async move {
            rt::time::sleep(Duration::from_secs(60)).await;

            wishes_import_infos.remove(job_id).await;
        });
    });

    Ok(HttpResponse::Ok().json(WishesImport { uid, job_id }))
}

async fn import_wishes(
    uid: i32,
    url: &Url,
    ignore_timestamps: bool,
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
                GiGachaType::Beginner => "100",
                GiGachaType::Standard => "200",
                GiGachaType::Character => "301",
                GiGachaType::Weapon => "302",
                GiGachaType::Chronicled => "500",
            },
        )])
        .finish();

    let mut set_all = database::gi::wishes::SetAll::default();

    let latest_timestamp = match gacha_type {
        GiGachaType::Beginner => {
            database::gi::wishes::beginner::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GiGachaType::Standard => {
            database::gi::wishes::standard::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GiGachaType::Character => {
            database::gi::wishes::character::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GiGachaType::Weapon => {
            database::gi::wishes::weapon::get_latest_timestamp_by_uid(uid, pool).await?
        }
        GiGachaType::Chronicled => {
            database::gi::wishes::chronicled::get_latest_timestamp_by_uid(uid, pool).await?
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

        let region_time_zone = match gacha_log.data.region.as_str() {
            "os_usa" => -5,
            "os_euro" => 1,
            _ => 8,
        };

        let tz = FixedOffset::east_opt(3600 * region_time_zone).unwrap();

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

            let item: i32 = if let Ok(id) =
                database::gi::characters_text::get_id_by_name(&entry.name, pool).await
            {
                id
            } else {
                database::gi::weapons_text::get_id_by_name(&entry.name, pool).await?
            };

            let mut character = (entry.item_type == "Character").then_some(item);
            let mut weapon = (entry.item_type == "Weapon").then_some(item);

            if character.is_none() && weapon.is_none() {
                if item >= 10000000 {
                    character = Some(item);
                } else {
                    weapon = Some(item);
                }
            }

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.weapon.push(weapon);
            set_all.timestamp.push(timestamp);
            set_all.official.push(true);

            match gacha_type {
                GiGachaType::Beginner => info.lock().await.beginner += 1,
                GiGachaType::Standard => info.lock().await.standard += 1,
                GiGachaType::Character => info.lock().await.character += 1,
                GiGachaType::Weapon => info.lock().await.weapon += 1,
                GiGachaType::Chronicled => info.lock().await.chronicled += 1,
            }
        }
    }

    let pulls = crate::gacha::imports::normalize_gi_set(gacha_type, &set_all)?;
    let batch = crate::gacha::imports::ImportBatch::new(
        pulls,
        crate::gacha::imports::ImportPolicy::official(),
    )?;
    crate::gacha::imports::persist_batch_in_transaction(&batch, pool).await?;

    Ok(())
}
