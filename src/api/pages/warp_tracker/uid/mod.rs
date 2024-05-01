use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams},
    database, GachaType,
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
    pull_4: usize,
    pull_5: usize,
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
            pull_4: 0,
            pull_5: 0,
        }
    }
}

#[derive(Serialize)]
struct WarpTracker {
    standard: Warps,
    departure: Warps,
    special: Warps,
    lc: Warps,
    count: usize,
    jades: usize,
}

#[derive(Default, Serialize)]
struct Warps {
    warps: Vec<Warp>,
    probability: f64,
    count: usize,
    jades: usize,
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
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let warps = database::get_warps_by_uid(*uid, language_params.lang, &pool).await?;

    let mut standard = Warps::default();
    let mut departure = Warps::default();
    let mut special = Warps::default();
    let mut lc = Warps::default();

    let mut standard_pull = 0;
    let mut departure_pull = 0;
    let mut special_pull = 0;
    let mut lc_pull = 0;

    let mut standard_pull_4 = 0;
    let mut departure_pull_4 = 0;
    let mut special_pull_4 = 0;
    let mut lc_pull_4 = 0;

    let mut standard_pull_5 = 0;
    let mut departure_pull_5 = 0;
    let mut special_pull_5 = 0;
    let mut lc_pull_5 = 0;

    for warp in warps {
        let gacha_type = warp.gacha_type;

        let mut warp: Warp = warp.into();

        match gacha_type {
            GachaType::Departure => {
                departure_pull += 1;
                departure_pull_4 += 1;
                departure_pull_5 += 1;

                warp.pull = departure_pull;
                warp.pull_4 = departure_pull_4;
                warp.pull_5 = departure_pull_5;

                match warp.rarity {
                    4 => departure_pull_4 = 0,
                    5 => departure_pull_5 = 0,
                    _ => {}
                }

                departure.warps.push(warp);
            }
            GachaType::Standard => {
                standard_pull += 1;
                standard_pull_4 += 1;
                standard_pull_5 += 1;

                warp.pull = standard_pull;
                warp.pull_4 = standard_pull_4;
                warp.pull_5 = standard_pull_5;

                match warp.rarity {
                    4 => standard_pull_4 = 0,
                    5 => standard_pull_5 = 0,
                    _ => {}
                }

                standard.warps.push(warp);
            }
            GachaType::Special => {
                special_pull += 1;
                special_pull_4 += 1;
                special_pull_5 += 1;

                warp.pull = special_pull;
                warp.pull_4 = special_pull_4;
                warp.pull_5 = special_pull_5;

                match warp.rarity {
                    4 => special_pull_4 = 0,
                    5 => special_pull_5 = 0,
                    _ => {}
                }

                special.warps.push(warp);
            }
            GachaType::Lc => {
                lc_pull += 1;
                lc_pull_4 += 1;
                lc_pull_5 += 1;

                warp.pull = lc_pull;
                warp.pull_4 = lc_pull_4;
                warp.pull_5 = lc_pull_5;

                match warp.rarity {
                    4 => lc_pull_4 = 0,
                    5 => lc_pull_5 = 0,
                    _ => {}
                }

                lc.warps.push(warp);
            }
        }
    }

    standard.probability = if standard_pull_5 < 89 {
        0.6 + 6.0 * standard_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    special.probability = if special_pull_5 < 89 {
        0.6 + 6.0 * special_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    lc.probability = if lc_pull_5 < 79 {
        0.8 + 7.0 * lc_pull_5.saturating_sub(64) as f64
    } else {
        100.0
    };

    departure.count = departure.warps.len();
    standard.count = standard.warps.len();
    special.count = special.warps.len();
    lc.count = lc.warps.len();

    departure.jades = departure.count * 160;
    standard.jades = standard.count * 160;
    special.jades = special.count * 160;
    lc.jades = lc.count * 160;

    let count = standard.count + departure.count + special.count + lc.count;
    let jades = standard.jades + departure.jades + special.jades + lc.jades;

    let warp_tracker = WarpTracker {
        standard,
        departure,
        special,
        lc,
        count,
        jades,
    };

    Ok(HttpResponse::Ok().json(warp_tracker))
}
