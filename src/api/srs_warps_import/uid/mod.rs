use std::io::BufReader;

use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::DateTime;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, File},
    database, GachaType,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "srs-warps-import/{uid}")),
    paths(post_srs_warps_import)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_srs_warps_import);
}

#[derive(serde::Deserialize)]
struct Srs {
    data: Data,
}

#[derive(serde::Deserialize)]
struct Data {
    stores: Stores,
}

#[derive(serde::Deserialize)]
struct Stores {
    #[serde(rename = "1-warp-v2")]
    warps: Warps,
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
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/srs-warps-import")]
async fn post_srs_warps_import(
    session: Session,
    uid: web::Path<i32>,
    file: MultipartForm<File>,
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

    let srs: Srs = serde_json::from_reader(BufReader::new(&file.file.file))?;

    import_warps(
        *uid,
        &srs.data.stores.warps.standard,
        GachaType::Standard,
        &pool,
    )
    .await?;
    import_warps(
        *uid,
        &srs.data.stores.warps.departure,
        GachaType::Departure,
        &pool,
    )
    .await?;
    import_warps(
        *uid,
        &srs.data.stores.warps.special,
        GachaType::Special,
        &pool,
    )
    .await?;
    import_warps(*uid, &srs.data.stores.warps.lc, GachaType::Lc, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn import_warps(
    uid: i32,
    warps: &[Warp],
    gacha_type: GachaType,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    for warp in warps {
        let id = warp.uid.parse::<i64>().unwrap();
        let (character, light_cone) = if warp.item_id < 2000 {
            (Some(warp.item_id), None)
        } else {
            (None, Some(warp.item_id))
        };

        let timestamp = DateTime::from_timestamp_millis(warp.timestamp).unwrap();

        // FIXME: Temp
        database::delete_warp_by_id_and_timestamp(id, timestamp + chrono::Duration::hours(5), pool)
            .await?;
        database::delete_warp_by_id_and_timestamp(id, timestamp + chrono::Duration::hours(6), pool)
            .await?;
        database::delete_warp_by_id_and_timestamp(
            id,
            timestamp + chrono::Duration::hours(13),
            pool,
        )
        .await?;
        // FIXME: Temp

        let warp = database::DbWarp {
            id,
            uid,
            gacha_type,
            character,
            light_cone,
            name: None,
            rarity: None,
            timestamp,
            official: false,
        };

        database::set_warp(&warp, pool).await?;
    }

    Ok(())
}
