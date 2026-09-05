use crate::gacha::imports::{ImportBatch, NormalizedPull, PullItem, PullPool, PullProvenance};
use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, GachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "srgf-warps-import")),
    paths(post_srgf_warps_import),
    components(schemas(Data))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_srgf_warps_import);
}

#[derive(Deserialize, utoipa::ToSchema)]
struct Data {
    data: String,
}

#[derive(Deserialize)]
struct Srgf {
    info: Info,
    list: Vec<Entry>,
}

#[derive(Deserialize)]
struct Info {
    uid: String,
    region_time_zone: i32,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    gacha_type: String,
    item_id: String,
    time: String,
}

struct ParsedWarp {
    id: i64,
    item_id: i32,
    time: DateTime<Utc>,
}

#[utoipa::path(
    tag = "srgf-warps-import",
    post,
    path = "/api/srgf-warps-import",
    request_body = Data,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/srgf-warps-import")]
async fn post_srgf_warps_import(
    session: Session,
    data: web::Json<Data>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(srgf) = serde_json::from_str::<Srgf>(&data.data) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(uid) = srgf.info.uid.parse() else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let (admin, allowed) = crate::api::users::verified_or_admin(&username, uid, &pool).await?;

    if !allowed {
        return Ok(HttpResponse::Forbidden().finish());
    }

    mihomo::ensure_row(uid, &pool).await?;

    let mut warps_map: HashMap<_, Vec<ParsedWarp>> = HashMap::new();
    let Some(tz) = srgf
        .info
        .region_time_zone
        .checked_mul(3600)
        .and_then(FixedOffset::east_opt)
    else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    for entry in &srgf.list {
        let gacha_type = match entry.gacha_type.as_str() {
            "1" => GachaType::Standard,
            "2" => GachaType::Departure,
            "11" => GachaType::Special,
            "12" => GachaType::Lc,
            "21" => GachaType::Collab,
            "22" => GachaType::CollabLc,
            _ => return Ok(HttpResponse::BadRequest().finish()),
        };

        let Some(time) = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")
            .ok()
            .and_then(|time| time.and_local_timezone(tz).single())
            .map(|time| time.to_utc())
        else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        let (Ok(id), Ok(item_id)) = (entry.id.parse(), entry.item_id.parse()) else {
            return Ok(HttpResponse::BadRequest().finish());
        };

        warps_map
            .entry(gacha_type)
            .or_default()
            .push(ParsedWarp { id, item_id, time });
    }

    let mut pulls = Vec::new();

    for gacha_type in GachaType::iter() {
        let Some(warps) = warps_map.get(&gacha_type) else {
            continue;
        };

        let earliest_timestamp =
            database::warps::get_earliest_timestamp_by_uid_by_pool(gacha_type, uid, &pool).await?;

        let count = database::warps::get_count_by_uid_by_pool(gacha_type, uid, &pool).await?;

        if count as usize + warps.len() >= 50000 {
            return Ok(HttpResponse::BadRequest().finish());
        }

        for warp in warps.iter() {
            let timestamp = warp.time;

            // File order is significant: stop this pool at its first overlap.
            if !admin {
                if let Some(earliest_timestamp) = earliest_timestamp {
                    if timestamp >= earliest_timestamp {
                        break;
                    }
                }
            }

            let id = warp.id;
            let item = if warp.item_id < 2000 {
                PullItem::Character(warp.item_id)
            } else {
                PullItem::LightCone(warp.item_id)
            };
            pulls.push(NormalizedPull {
                uid,
                id,
                pool: PullPool::Hsr(gacha_type),
                item,
                timestamp,
                provenance: PullProvenance::Unofficial,
            });
        }
    }

    let Ok(batch) = ImportBatch::new(pulls, PullProvenance::Unofficial) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    crate::gacha::imports::persist_batch_in_transaction(&batch, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
