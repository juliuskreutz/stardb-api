use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, GachaType, Language};

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
            ..std::default::Default::default()
        };

        database::mihomo::set(&db_mihomo, &pool).await?;
    }

    let timestamp_offset = chrono::Duration::hours(match uid.to_string().chars().next() {
        Some('6') => -5,
        Some('7') => 1,
        _ => 8,
    });

    let pom: Pom = serde_json::from_str(&params.data)?;

    let mut set_all_departure = database::warps::SetAll::default();
    let mut set_all_standard = database::warps::SetAll::default();
    let mut set_all_special = database::warps::SetAll::default();
    let mut set_all_lc = database::warps::SetAll::default();
    let mut set_all_collab = database::warps::SetAll::default();
    let mut set_all_collab_lc = database::warps::SetAll::default();

    for (warps, gacha_type) in [
        (&pom.default.departure, GachaType::Departure),
        (&pom.default.standard, GachaType::Standard),
        (&pom.default.special, GachaType::Special),
        (&pom.default.lc, GachaType::Lc),
    ] {
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

        for warp in warps {
            let timestamp = NaiveDateTime::parse_from_str(&warp.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                - timestamp_offset;

            if !admin {
                if let Some(earliest_timestamp) = earliest_timestamp {
                    if timestamp >= earliest_timestamp {
                        break;
                    }
                }
            }

            let id = warp.id.parse::<i64>().unwrap();
            let item_id = warp.item_id.parse().unwrap();
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
