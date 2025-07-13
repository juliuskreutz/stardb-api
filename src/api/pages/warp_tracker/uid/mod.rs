use std::collections::HashMap;

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
    win: Option<WinType>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WinType {
    Win,
    Loss,
    Guarantee,
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
            win: None,
        }
    }
}

#[derive(Serialize)]
struct WarpTracker {
    standard: Warps,
    departure: Warps,
    special: Warps,
    lc: Warps,
    collab: Warps,
    collab_lc: Warps,
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
    luck_4: f64,
    luck_5: f64,
    win_stats: Option<WinStats>,
    global_stats: Option<GlobalStats>,
}

#[derive(Serialize)]
struct WinStats {
    win_rate: f64,
    win_streak: i32,
    loss_streak: i32,
}

#[derive(Serialize)]
struct GlobalStats {
    count_percentile: f64,
    luck_4_percentile: f64,
    luck_5_percentile: f64,
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

    let mut forbidden = database::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if database::admins::exists(&username, &pool).await? {
                forbidden = false;
            } else if let Ok(connection) =
                database::connections::get_by_uid_and_username(uid, &username, &pool).await
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

    let mut banners: HashMap<_, Vec<_>> = HashMap::new();

    for banner in database::banners::get_all(&pool).await? {
        if let Some(character) = banner.character {
            banners
                .entry(character)
                .or_default()
                .push(banner.start..banner.end);
        }

        if let Some(light_cone) = banner.light_cone {
            banners
                .entry(light_cone)
                .or_default()
                .push(banner.start..banner.end);
        }
    }

    // region Departure
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
    // endregion Departure

    // region Standard
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
    // endregion Standard

    // region Special
    let mut special = Warps::default();
    let mut special_pull = 0;
    let mut special_pull_4 = 0;
    let mut special_pull_5 = 0;
    let mut guarantee = false;

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
            5 => {
                special_pull_5 = 0;

                warp.win = if guarantee {
                    guarantee = false;

                    Some(WinType::Guarantee)
                } else if banners
                    .get(&warp.item_id)
                    .map(|v| v.iter().any(|r| r.contains(&warp.timestamp)))
                    .unwrap_or_default()
                {
                    Some(WinType::Win)
                } else {
                    guarantee = true;

                    Some(WinType::Loss)
                };
            }
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
    // endregion Special

    // region Lc
    let mut lc = Warps::default();
    let mut lc_pull = 0;
    let mut lc_pull_4 = 0;
    let mut lc_pull_5 = 0;
    let mut guarantee = false;

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
            5 => {
                lc_pull_5 = 0;

                warp.win = if guarantee {
                    guarantee = false;

                    Some(WinType::Guarantee)
                } else if banners
                    .get(&warp.item_id)
                    .map(|v| v.iter().any(|r| r.contains(&warp.timestamp)))
                    .unwrap_or_default()
                {
                    Some(WinType::Win)
                } else {
                    guarantee = true;

                    Some(WinType::Loss)
                };
            }
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
    // endregion Lc

    // region Collab
    let mut collab = Warps::default();
    let mut collab_pull = 0;
    let mut collab_pull_4 = 0;
    let mut collab_pull_5 = 0;
    let mut collab_guarantee = false;

    for warp in database::warps::collab::get_by_uid(uid, language, &pool).await? {
        let mut warp: Warp = warp.into();

        collab_pull += 1;
        collab_pull_4 += 1;
        collab_pull_5 += 1;

        warp.pull = collab_pull;
        warp.pull_4 = collab_pull_4;
        warp.pull_5 = collab_pull_5;

        match warp.rarity {
            4 => collab_pull_4 = 0,
            5 => {
                collab_pull_5 = 0;

                warp.win = if collab_guarantee {
                    collab_guarantee = false;
                    Some(WinType::Guarantee)
                } else if banners
                    .get(&warp.item_id)
                    .map(|v| v.iter().any(|r| r.contains(&warp.timestamp)))
                    .unwrap_or_default()
                {
                    Some(WinType::Win)
                } else {
                    collab_guarantee = true;
                    Some(WinType::Loss)
                };
            }
            _ => {}
        }

        collab.warps.push(warp);
    }

    collab.pull_4 = collab_pull_4;
    collab.max_pull_4 = 10;
    collab.probability_4 = if collab_pull_4 < 9 { 5.1 } else { 100.0 };

    collab.pull_5 = collab_pull_5;
    collab.max_pull_5 = 90;
    collab.probability_5 = if collab_pull_5 < 89 {
        0.6 + 6.0 * collab_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    collab.count = collab.warps.len();
    // endregion Collab

    // region Collab LC
    let mut collab_lc = Warps::default();
    let mut collab_lc_pull = 0;
    let mut collab_lc_pull_4 = 0;
    let mut collab_lc_pull_5 = 0;
    let mut collab_lc_guarantee = false;

    for warp in database::warps::collab_lc::get_by_uid(uid, language, &pool).await? {
        let mut warp: Warp = warp.into();

        collab_lc_pull += 1;
        collab_lc_pull_4 += 1;
        collab_lc_pull_5 += 1;

        warp.pull = collab_lc_pull;
        warp.pull_4 = collab_lc_pull_4;
        warp.pull_5 = collab_lc_pull_5;

        match warp.rarity {
            4 => collab_lc_pull_4 = 0,
            5 => {
                collab_lc_pull_5 = 0;

                warp.win = if collab_lc_guarantee {
                    collab_lc_guarantee = false;
                    Some(WinType::Guarantee)
                } else if banners
                    .get(&warp.item_id)
                    .map(|v| v.iter().any(|r| r.contains(&warp.timestamp)))
                    .unwrap_or_default()
                {
                    Some(WinType::Win)
                } else {
                    collab_lc_guarantee = true;
                    Some(WinType::Loss)
                };
            }
            _ => {}
        }

        collab_lc.warps.push(warp);
    }

    collab_lc.pull_4 = collab_lc_pull_4;
    collab_lc.max_pull_4 = 10;
    collab_lc.probability_4 = if collab_lc_pull_4 < 9 { 6.6 } else { 100.0 };

    collab_lc.pull_5 = collab_lc_pull_5;
    collab_lc.max_pull_5 = 80;
    collab_lc.probability_5 = if collab_lc_pull_5 < 79 {
        0.8 + 7.0 * collab_lc_pull_5.saturating_sub(64) as f64
    } else {
        100.0
    };

    collab_lc.count = collab_lc.warps.len();
    // endregion Collab LC

    // region Stats
    if let Some(stats) = database::warps_stats::standard::get_by_uid(uid, &pool).await? {
        let global_stats = database::warps_stats_global::standard::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        standard.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats: None,
            global_stats,
        });
    }

    if let Some(stats) = database::warps_stats::special::get_by_uid(uid, &pool).await? {
        let win_stats = Some(WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        });

        let global_stats = database::warps_stats_global::special::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        special.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats,
            global_stats,
        });
    }

    if let Some(stats) = database::warps_stats::lc::get_by_uid(uid, &pool).await? {
        let win_stats = Some(WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        });

        let global_stats = database::warps_stats_global::lc::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        lc.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats,
            global_stats,
        });
    }

    if let Some(stats) = database::warps_stats::collab::get_by_uid(uid, &pool).await? {
        let win_stats = Some(WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        });

        let global_stats = database::warps_stats_global::collab::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        collab.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats,
            global_stats,
        });
    }

    if let Some(stats) = database::warps_stats::collab_lc::get_by_uid(uid, &pool).await? {
        let win_stats = Some(WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        });

        let global_stats = database::warps_stats_global::collab_lc::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        collab_lc.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats,
            global_stats,
        });
    }
    // endregion Stats

    let warp_tracker = WarpTracker {
        standard,
        departure,
        special,
        lc,
        collab,
        collab_lc,
        name,
    };

    Ok(HttpResponse::Ok().json(warp_tracker))
}
