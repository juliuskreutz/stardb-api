use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use rand::seq::IndexedRandom as _;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, GachaType, Language};

#[derive(OpenApi)]
#[openapi(
    tags((name = "srs-warps-import/{uid}")),
    paths(post_srs_warps_import),
    components(schemas(SrsWarpsImportParams)),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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

    let admin = database::admins::exists(&username, &pool).await?;

    let allowed = admin
        || database::connections::get_by_username(&username, &pool)
            .await?
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.verified)
            .unwrap_or_default();

    if !allowed {
        return Ok(HttpResponse::Forbidden().finish());
    }

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

    let mut warps_map: HashMap<_, Vec<ParsedWarp>> = HashMap::new();

    let mut reader = csv::Reader::from_reader(params.data.as_bytes());
    for warp in reader.deserialize() {
        let warp: Warp = warp?;

        let time = DateTime::parse_from_rfc3339(&warp.time).unwrap().to_utc();

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

    let mut set_all_departure = database::warps::SetAll::default();
    let mut set_all_standard = database::warps::SetAll::default();
    let mut set_all_special = database::warps::SetAll::default();
    let mut set_all_lc = database::warps::SetAll::default();
    let mut set_all_collab = database::warps::SetAll::default();
    let mut set_all_collab_lc = database::warps::SetAll::default();

    for (warps, gacha_type) in [
        (&warps_map.get(&1), GachaType::Standard),
        (&warps_map.get(&2), GachaType::Departure),
        (&warps_map.get(&11), GachaType::Special),
        (&warps_map.get(&12), GachaType::Lc),
        (&warps_map.get(&21), GachaType::Collab),
        (&warps_map.get(&22), GachaType::CollabLc),
    ] {
        let Some(warps) = warps else {
            continue;
        };

        let count = match gacha_type {
            GachaType::Departure => {
                database::warps::departure::get_count_by_uid(uid, &pool).await?
            }
            GachaType::Standard => database::warps::standard::get_count_by_uid(uid, &pool).await?,
            GachaType::Special => database::warps::special::get_count_by_uid(uid, &pool).await?,
            GachaType::Lc => database::warps::lc::get_count_by_uid(uid, &pool).await?,
            GachaType::Collab => database::warps::collab::get_count_by_uid(uid, &pool).await?,
            GachaType::CollabLc => database::warps::collab_lc::get_count_by_uid(uid, &pool).await?,
        };

        if count as usize + warps.len() >= 50000 {
            return Ok(HttpResponse::BadRequest().finish());
        }

        let earliest_timestamp = match gacha_type {
            GachaType::Departure => {
                database::warps::departure::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GachaType::Standard => {
                database::warps::standard::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GachaType::Special => {
                database::warps::special::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GachaType::Lc => database::warps::lc::get_earliest_timestamp_by_uid(uid, &pool).await?,
            GachaType::Collab => {
                database::warps::collab::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GachaType::CollabLc => {
                database::warps::collab_lc::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
        };

        let mut pity = 0;

        for warp in warps.iter() {
            let timestamp = warp.time;

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
                    item_id = *light_cone_4_ids.choose(&mut rand::rng()).unwrap();
                    rarity = 4;
                } else {
                    item_id = *light_cone_3_ids.choose(&mut rand::rng()).unwrap();
                    rarity = 3;
                }
            }

            if rarity == 4 {
                pity = 0;
            } else {
                pity += 1;
            }

            let id = warp.id;

            let (character, light_cone) = if item_id < 2000 {
                (Some(item_id), None)
            } else {
                (None, Some(item_id))
            };

            let set_all = match gacha_type {
                GachaType::Departure => &mut set_all_departure,
                GachaType::Standard => &mut set_all_standard,
                GachaType::Special => &mut set_all_special,
                GachaType::Lc => &mut set_all_lc,
                GachaType::Collab => &mut set_all_collab,
                GachaType::CollabLc => &mut set_all_collab_lc,
            };

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.light_cone.push(light_cone);
            set_all.timestamp.push(timestamp);
            set_all.official.push(false);
        }
    }

    database::warps::departure::set_all(&set_all_departure, &pool).await?;
    database::warps::standard::set_all(&set_all_standard, &pool).await?;
    database::warps::special::set_all(&set_all_special, &pool).await?;
    database::warps::lc::set_all(&set_all_lc, &pool).await?;
    database::warps::collab::set_all(&set_all_collab, &pool).await?;
    database::warps::collab_lc::set_all(&set_all_collab_lc, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
