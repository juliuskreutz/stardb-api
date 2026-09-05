use crate::gacha::imports::{ImportBatch, NormalizedPull, PullItem, PullPool, PullProvenance};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, GachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "pom-warps-import/{uid}")),
    paths(post_pom_warps_import),
    components(schemas(PomWarpsImportParams)),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_pom_warps_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct PomWarpsImportParams {
    data: String,
}

#[derive(serde::Deserialize)]
struct Pom {
    default: Default,
}

#[derive(serde::Deserialize)]
struct Default {
    #[serde(rename = "beginner")]
    departure: Vec<Warp>,
    #[serde(rename = "standard")]
    standard: Vec<Warp>,
    #[serde(rename = "character")]
    special: Vec<Warp>,
    #[serde(rename = "lightcone")]
    lc: Vec<Warp>,
}

#[derive(serde::Deserialize)]
struct Warp {
    id: String,
    #[serde(rename = "itemId")]
    item_id: String,
    time: String,
}

#[utoipa::path(
    tag = "pom-warps-import/{uid}",
    post,
    path = "/api/pom-warps-import/{uid}",
    request_body = PomWarpsImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/pom-warps-import/{uid}")]
async fn post_pom_warps_import(
    session: Session,
    uid: web::Path<i32>,
    params: web::Json<PomWarpsImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uid = *uid;

    let (admin, allowed) = crate::api::users::verified_or_admin(&username, uid, &pool).await?;

    if !allowed {
        return Ok(HttpResponse::Forbidden().finish());
    }

    mihomo::ensure_row(uid, &pool).await?;

    let timestamp_offset = chrono::Duration::hours(match uid.to_string().chars().next() {
        Some('6') => -5,
        Some('7') => 1,
        _ => 8,
    });

    let Ok(pom) = serde_json::from_str::<Pom>(&params.data) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut pulls = Vec::new();

    for (warps, gacha_type) in [
        (&pom.default.departure, GachaType::Departure),
        (&pom.default.standard, GachaType::Standard),
        (&pom.default.special, GachaType::Special),
        (&pom.default.lc, GachaType::Lc),
    ] {
        let count = database::warps::get_count_by_uid_by_pool(gacha_type, uid, &pool).await?;

        if count as usize + warps.len() >= 50000 {
            return Ok(HttpResponse::BadRequest().finish());
        }

        let earliest_timestamp =
            database::warps::get_earliest_timestamp_by_uid_by_pool(gacha_type, uid, &pool).await?;

        for warp in warps {
            let Some(timestamp) = NaiveDateTime::parse_from_str(&warp.time, "%Y-%m-%d %H:%M:%S")
                .ok()
                .and_then(|time| time.and_utc().checked_sub_signed(timestamp_offset))
            else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            if !admin {
                if let Some(earliest_timestamp) = earliest_timestamp {
                    if timestamp >= earliest_timestamp {
                        break;
                    }
                }
            }

            let Ok(id) = warp.id.parse::<i64>() else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            let Ok(item_id) = warp.item_id.parse() else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            let item = if item_id < 2000 {
                PullItem::Character(item_id)
            } else {
                PullItem::LightCone(item_id)
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
