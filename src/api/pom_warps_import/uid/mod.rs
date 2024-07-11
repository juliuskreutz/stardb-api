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
    #[serde(rename = "standard")]
    standard: Vec<Warp>,
    #[serde(rename = "beginner")]
    departure: Vec<Warp>,
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

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;

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

    let mut warp_id = Vec::new();
    let mut warp_uid = Vec::new();
    let mut warp_character = Vec::new();
    let mut warp_light_cone = Vec::new();
    let mut warp_gacha_type = Vec::new();
    let mut warp_timestamp = Vec::new();
    let mut warp_official = Vec::new();

    for (warps, gacha_type) in [
        (&pom.default.standard, GachaType::Standard),
        (&pom.default.departure, GachaType::Departure),
        (&pom.default.special, GachaType::Special),
        (&pom.default.lc, GachaType::Lc),
    ] {
        for warp in warps {
            let id = warp.id.parse::<i64>().unwrap();
            let item_id = warp.item_id.parse().unwrap();
            let (character, light_cone) = if item_id < 2000 {
                (Some(item_id), None)
            } else {
                (None, Some(item_id))
            };

            let timestamp = NaiveDateTime::parse_from_str(&warp.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                - timestamp_offset;

            // FIXME: Temp
            database::delete_warp_by_id_and_timestamp(
                id,
                timestamp + chrono::Duration::hours(1),
                &pool,
            )
            .await?;
            database::delete_warp_by_id_and_timestamp(
                id,
                timestamp + chrono::Duration::hours(6),
                &pool,
            )
            .await?;
            database::delete_warp_by_id_and_timestamp(
                id,
                timestamp + chrono::Duration::hours(13),
                &pool,
            )
            .await?;
            // FIXME: Temp

            warp_id.push(id);
            warp_uid.push(uid);
            warp_character.push(character);
            warp_light_cone.push(light_cone);
            warp_gacha_type.push(gacha_type);
            warp_timestamp.push(timestamp);
            warp_official.push(false);
        }
    }

    database::set_all_warps(
        &warp_id,
        &warp_uid,
        &warp_gacha_type,
        &warp_character,
        &warp_light_cone,
        &warp_timestamp,
        &warp_official,
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}
