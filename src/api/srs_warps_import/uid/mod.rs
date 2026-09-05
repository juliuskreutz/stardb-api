use crate::gacha::imports::{ImportBatch, NormalizedPull, PullItem, PullPool, PullProvenance};
use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use rand::seq::IndexedRandom as _;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, GachaType, Language};

#[derive(OpenApi)]
#[openapi(
    tags((name = "srs-warps-import/{uid}")),
    paths(post_srs_warps_import),
    components(schemas(SrsWarpsImportParams)),
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_srs_warps_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct SrsWarpsImportParams {
    data: String,
}

#[derive(serde::Deserialize)]
struct Warp {
    uid: i64,
    id: i32,
    rarity: i32,
    time: String,
    #[serde(rename = "type")]
    gacha_type: i32,
}

struct ParsedWarp {
    id: i64,
    rarity: i32,
    item_id: i32,
    time: DateTime<Utc>,
}

#[utoipa::path(
    tag = "srs-warps-import/{uid}",
    post,
    path = "/api/srs-warps-import/{uid}",
    request_body = SrsWarpsImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not verified"),
    )
)]
#[post("/api/srs-warps-import/{uid}")]
/// Imports SRS CSV as unofficial pulls for an admin or verified HSR connection.
/// Preserves source order and stops each non-admin pool at its first overlap; unknown
/// item IDs use rarity-based catalog samples. Invalid CSV/time/batches return 400.
async fn post_srs_warps_import(
    session: Session,
    uid: web::Path<i32>,
    params: web::Json<SrsWarpsImportParams>,
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

    let mut warps_map: HashMap<_, Vec<ParsedWarp>> = HashMap::new();

    let mut reader = csv::Reader::from_reader(params.data.as_bytes());
    for warp in reader.deserialize() {
        let Ok(warp) = warp else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        let warp: Warp = warp;

        let Ok(time) = DateTime::parse_from_rfc3339(&warp.time) else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        let time = time.to_utc();

        warps_map
            .entry(warp.gacha_type)
            .or_default()
            .push(ParsedWarp {
                id: warp.uid,
                rarity: warp.rarity,
                item_id: warp.id,
                time,
            });
    }

    let db_light_cones = database::light_cones::get_all(Language::En, &pool).await?;

    let light_cone_3_ids: Vec<i32> = db_light_cones
        .iter()
        .filter_map(|lc| (lc.rarity == 3).then_some(lc.id))
        .collect();

    let light_cone_4_ids: Vec<i32> = db_light_cones
        .iter()
        .filter_map(|lc| (lc.rarity == 4).then_some(lc.id))
        .collect();

    let mut pulls = Vec::new();

    for gacha_type in GachaType::iter() {
        let warps = warps_map.get(&gacha_type.id());
        let Some(warps) = warps else {
            continue;
        };

        let count = database::warps::get_count_by_uid_by_pool(gacha_type, uid, &pool).await?;

        if count as usize + warps.len() >= 50000 {
            return Ok(HttpResponse::BadRequest().finish());
        }

        let earliest_timestamp =
            database::warps::get_earliest_timestamp_by_uid_by_pool(gacha_type, uid, &pool).await?;

        let mut pity = 0;

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

            let mut item_id = warp.item_id;
            let mut rarity = warp.rarity;

            if item_id == 0 {
                if pity >= 9 {
                    item_id = *light_cone_4_ids
                        .choose(&mut rand::rng())
                        .ok_or_else(|| anyhow::anyhow!("missing light cone catalog"))?;
                    rarity = 4;
                } else {
                    item_id = *light_cone_3_ids
                        .choose(&mut rand::rng())
                        .ok_or_else(|| anyhow::anyhow!("missing light cone catalog"))?;
                    rarity = 3;
                }
            }

            if rarity == 4 {
                pity = 0;
            } else {
                pity += 1;
            }

            let id = warp.id;

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
