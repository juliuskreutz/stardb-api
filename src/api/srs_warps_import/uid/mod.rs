use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::DateTime;
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
    profile: i32,
}

#[derive(serde::Deserialize)]
struct Srs {
    data: Data,
}

#[derive(serde::Deserialize)]
struct Data {
    stores: HashMap<String, serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct Warps {
    #[serde(rename = "items_1")]
    standard: Vec<Warp>,
    #[serde(rename = "items_2")]
    departure: Vec<Warp>,
    #[serde(rename = "items_11")]
    special: Vec<Warp>,
    #[serde(rename = "items_12")]
    lc: Vec<Warp>,
}

#[derive(serde::Deserialize)]
struct Warp {
    uid: String,
    #[serde(rename = "itemId")]
    item_id: i32,
    timestamp: i64,
}

#[utoipa::path(
    tag = "srs-warps-import/{uid}",
    post,
    path = "/api/srs-warps-import/{uid}",
    request_body = SrsWarpsImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
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
        let db_mihomo = database::mihomo::DbMihomo {
            uid,
            ..Default::default()
        };

        database::mihomo::set(&db_mihomo, &pool).await?;
    }

    let srs: Srs = serde_json::from_str(&params.data)?;

    let profile = params.profile;
    let warps: Warps =
        serde_json::from_value(srs.data.stores[&format!("{profile}_warp-v2")].clone())?;

    let mut warp_id = Vec::new();
    let mut warp_uid = Vec::new();
    let mut warp_character = Vec::new();
    let mut warp_light_cone = Vec::new();
    let mut warp_gacha_type = Vec::new();
    let mut warp_timestamp = Vec::new();
    let mut warp_official = Vec::new();

    for (warps, gacha_type) in [
        (&warps.standard, GachaType::Standard),
        (&warps.departure, GachaType::Departure),
        (&warps.special, GachaType::Special),
        (&warps.lc, GachaType::Lc),
    ] {
        for warp in warps {
            let id = warp.uid.parse::<i64>().unwrap();
            let (character, light_cone) = if warp.item_id < 2000 {
                (Some(warp.item_id), None)
            } else {
                (None, Some(warp.item_id))
            };

            let timestamp = DateTime::from_timestamp_millis(warp.timestamp).unwrap();

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
