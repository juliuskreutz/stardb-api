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
    pull: usize,
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
            pull: 0,
        }
    }
}

#[derive(Serialize)]
struct WarpTracker {
    count: usize,
    standard_probability: f64,
    special_probability: f64,
    lc_probability: f64,
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

    let mut standard_pull = 0;
    let mut departure_pull = 0;
    let mut special_pull = 0;
    let mut lc_pull = 0;

    for warp in warps {
        let gacha_type = warp.gacha_type.clone();

        let mut warp: Warp = warp.into();

        match gacha_type.as_str() {
            "standard" => {
                standard_pull += 1;

                warp.pull = standard_pull;

                if warp.rarity == 5 {
                    standard_pull = 0;
                }

                standard.push(warp);
            }
            "departure" => {
                departure_pull += 1;

                warp.pull = departure_pull;

                if warp.rarity == 5 {
                    departure_pull = 0;
                }

                departure.push(warp);
            }
            "special" => {
                special_pull += 1;

                warp.pull = special_pull;

                if warp.rarity == 5 {
                    special_pull = 0;
                }

                special.push(warp);
            }
            "lc" => {
                lc_pull += 1;

                warp.pull = lc_pull;

                if warp.rarity == 5 {
                    lc_pull = 0;
                }

                lc.push(warp);
            }
            _ => {}
        }
    }

    let standard_probability = if standard_pull < 89 {
        0.6 + 6.0 * standard_pull.saturating_sub(72) as f64
    } else {
        100.0
    };

    let special_probability = if special_pull < 89 {
        0.6 + 6.0 * special_pull.saturating_sub(72) as f64
    } else {
        100.0
    };

    let lc_probability = if lc_pull < 79 {
        0.8 + 7.0 * lc_pull.saturating_sub(64) as f64
    } else {
        100.0
    };

    let count = standard.len() + departure.len() + special.len() + lc.len();

    let warp_tracker = WarpTracker {
        count,
        standard_probability,
        special_probability,
        lc_probability,
        standard,
        departure,
        special,
        lc,
    };

    Ok(HttpResponse::Ok().json(warp_tracker))
}
