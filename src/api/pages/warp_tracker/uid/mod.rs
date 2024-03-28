use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(paths(get_warp_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warp_tracker);
}

#[derive(Serialize)]
struct Warp {
    r#type: WarpType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

impl From<database::DbWarp> for Warp {
    fn from(warp: database::DbWarp) -> Self {
        let r#type = if warp.character.is_some() {
            WarpType::Character
        } else {
            WarpType::LightCone
        };

        Self {
            r#type,
            id: warp.id.to_string(),
            name: warp.name.unwrap(),
            rarity: warp.rarity.unwrap(),
            item_id: warp.character.or(warp.light_cone).unwrap(),
            timestamp: warp.timestamp,
        }
    }
}

#[derive(Serialize)]
struct WarpTracker {
    count: usize,
    standard: Vec<Warp>,
    departure: Vec<Warp>,
    special: Vec<Warp>,
    lc: Vec<Warp>,
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/warp-tracker/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "WarpTracker"),
    )
)]
#[get("/api/pages/warp-tracker/{uid}", guard = "private")]
async fn get_warp_tracker(
    uid: web::Path<i64>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let warps = database::get_warps_by_uid(*uid, &language_params.lang.to_string(), &pool).await?;

    let mut standard = Vec::new();
    let mut departure = Vec::new();
    let mut special = Vec::new();
    let mut lc = Vec::new();

    for warp in warps {
        match warp.gacha_type.as_str() {
            "standard" => standard.push(warp.into()),
            "departure" => departure.push(warp.into()),
            "special" => special.push(warp.into()),
            "lc" => lc.push(warp.into()),
            _ => {}
        }
    }

    let count = standard.len() + departure.len() + special.len() + lc.len();

    let warp_tracker = WarpTracker {
        count,
        standard,
        departure,
        special,
        lc,
    };

    Ok(HttpResponse::Ok().json(warp_tracker))
}
