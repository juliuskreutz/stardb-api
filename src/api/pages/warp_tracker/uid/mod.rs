use actix_session::Session;
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

impl From<database::warps::DbWarp> for Warp {
    fn from(warp: database::warps::DbWarp) -> Self {
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
    name: String,
}

#[derive(Default, Serialize)]
struct Warps {
    warps: Vec<Warp>,
    probability_4: f64,
    probability_5: f64,
    pull_4: usize,
    pull_5: usize,
    max_pull_4: usize,
    max_pull_5: usize,
    count: usize,
    stats: Option<Stats>,
}

#[derive(Serialize)]
struct Stats {
    users: i32,
    count_percentile: f64,
    luck_4: f64,
    luck_4_percentile: f64,
    luck_5: f64,
    luck_5_percentile: f64,
    win_rate: Option<f64>,
    win_streak: Option<i32>,
    loss_streak: Option<i32>,
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
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::get_connections_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if let Ok(connection) =
                database::get_connection_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let language = language_params.lang;

    let name = database::mihomo::get_one_by_uid(uid, &pool).await?.name;

    // Departure
    let mut departure = Warps::default();
    let mut departure_pull = 0;
    let mut departure_pull_4 = 0;
    let mut departure_pull_5 = 0;

    for warp in database::warps::departure::get_by_uid(uid, language, &pool).await? {
        let mut warp: Warp = warp.into();

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

    departure.count = departure.warps.len();
    // Departure

    // Standard
    let mut standard = Warps::default();
    let mut standard_pull = 0;
    let mut standard_pull_4 = 0;
    let mut standard_pull_5 = 0;

    for warp in database::warps::standard::get_by_uid(uid, language, &pool).await? {
        let mut warp: Warp = warp.into();

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

    standard.pull_4 = standard_pull_4;
    standard.max_pull_4 = 10;
    standard.probability_4 = if standard_pull_4 < 9 { 5.1 } else { 100.0 };

    standard.pull_5 = standard_pull_5;
    standard.max_pull_5 = 90;
    standard.probability_5 = if standard_pull_5 < 89 {
        0.6 + 6.0 * standard_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    standard.count = standard.warps.len();
    // Standard

    // Special
    let mut special = Warps::default();
    let mut special_pull = 0;
    let mut special_pull_4 = 0;
    let mut special_pull_5 = 0;

    for warp in database::warps::special::get_by_uid(uid, language, &pool).await? {
        let mut warp: Warp = warp.into();

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

    special.pull_4 = special_pull_4;
    special.max_pull_4 = 10;
    special.probability_4 = if special_pull_4 < 9 { 5.1 } else { 100.0 };

    special.pull_5 = special_pull_5;
    special.max_pull_5 = 90;
    special.probability_5 = if special_pull_5 < 89 {
        0.6 + 6.0 * special_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    special.count = special.warps.len();
    // Special

    // Lc
    let mut lc = Warps::default();
    let mut lc_pull = 0;
    let mut lc_pull_4 = 0;
    let mut lc_pull_5 = 0;

    for warp in database::warps::lc::get_by_uid(uid, language, &pool).await? {
        let mut warp: Warp = warp.into();

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

    lc.pull_4 = lc_pull_4;
    lc.max_pull_4 = 10;
    lc.probability_4 = if lc_pull_4 < 9 { 6.6 } else { 100.0 };

    lc.pull_5 = lc_pull_5;
    lc.max_pull_5 = 80;
    lc.probability_5 = if lc_pull_5 < 79 {
        0.8 + 7.0 * lc_pull_5.saturating_sub(64) as f64
    } else {
        100.0
    };

    lc.count = lc.warps.len();
    // Lc

    if let Some(stats) = database::warps_stats_standard::get_by_uid(uid, &pool).await? {
        let users = database::warps_stats_standard::count(&pool).await? as i32;

        standard.stats = Some(Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_4,
            luck_4_percentile: stats.luck_4_percentile,
            luck_5: stats.luck_5,
            luck_5_percentile: stats.luck_5_percentile,
            win_rate: None,
            win_streak: None,
            loss_streak: None,
        });
    }

    if let Some(stats) = database::warps_stats_special::get_by_uid(uid, &pool).await? {
        let users = database::warps_stats_special::count(&pool).await? as i32;

        special.stats = Some(Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_4,
            luck_4_percentile: stats.luck_4_percentile,
            luck_5: stats.luck_5,
            luck_5_percentile: stats.luck_5_percentile,
            win_rate: Some(stats.win_rate),
            win_streak: Some(stats.win_streak),
            loss_streak: Some(stats.loss_streak),
        });
    }

    if let Some(stats) = database::warps_stats_lc::get_by_uid(uid, &pool).await? {
        let users = database::warps_stats_lc::count(&pool).await? as i32;

        lc.stats = Some(Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_4,
            luck_4_percentile: stats.luck_4_percentile,
            luck_5: stats.luck_5,
            luck_5_percentile: stats.luck_5_percentile,
            win_rate: Some(stats.win_rate),
            win_streak: Some(stats.win_streak),
            loss_streak: Some(stats.loss_streak),
        });
    }

    let warp_tracker = WarpTracker {
        standard,
        departure,
        special,
        lc,
        name,
    };

    Ok(HttpResponse::Ok().json(warp_tracker))
}
