use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
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
    time: String,
    #[serde(rename = "type")]
    gacha_type: i32,
}

struct ParsedWarp {
    id: i64,
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
                item_id: warp.id,
                time,
            });
    }

    let mut set_all_departure = database::warps::SetAll::default();
    let mut set_all_standard = database::warps::SetAll::default();
    let mut set_all_special = database::warps::SetAll::default();
    let mut set_all_lc = database::warps::SetAll::default();

    for (warps, gacha_type) in [
        (&warps_map.get(&1), GachaType::Standard),
        (&warps_map.get(&2), GachaType::Departure),
        (&warps_map.get(&11), GachaType::Special),
        (&warps_map.get(&12), GachaType::Lc),
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
        };

        for warp in warps.iter() {
            let timestamp = warp.time;

            if !admin {
                if let Some(earliest_timestamp) = earliest_timestamp {
                    if timestamp >= earliest_timestamp {
                        break;
                    }
                }
            }

            let id = warp.id;
            let (character, light_cone) = if warp.item_id < 2000 {
                (Some(warp.item_id), None)
            } else {
                (None, Some(warp.item_id))
            };

            let set_all = match gacha_type {
                GachaType::Departure => &mut set_all_departure,
                GachaType::Standard => &mut set_all_standard,
                GachaType::Special => &mut set_all_special,
                GachaType::Lc => &mut set_all_lc,
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

    calculate_stats_standard(uid, &pool).await?;
    calculate_stats_special(uid, &pool).await?;
    calculate_stats_lc(uid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn calculate_stats_standard(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let warps = database::warps::standard::get_infos_by_uid(uid, pool).await?;

    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    for warp in &warps {
        pull_4 += 1;
        pull_5 += 1;

        match warp.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;
            }
            _ => {}
        }
    }

    let luck_4 = sum_4 as f64 / count_4 as f64;
    let luck_5 = sum_5 as f64 / count_5 as f64;

    let stat = database::warps_stats::standard::DbWarpsStatStandard {
        uid,
        luck_4,
        luck_5,
    };
    database::warps_stats::standard::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_special(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let warps = database::warps::special::get_infos_by_uid(uid, pool).await?;

    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for warp in &warps {
        pull_4 += 1;
        pull_5 += 1;

        match warp.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [1209, 1004, 1101, 1211, 1104, 1107, 1003].contains(&warp.character.unwrap())
                    {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = sum_4 as f64 / count_4 as f64;
    let luck_5 = sum_5 as f64 / count_5 as f64;
    let win_rate = sum_win as f64 / count_win as f64;

    let stat = database::warps_stats::special::DbWarpsStatSpecial {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::special::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_lc(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let warps = database::warps::lc::get_infos_by_uid(uid, pool).await?;

    let mut pull_4 = 0;
    let mut sum_4 = 0;
    let mut count_4 = 0;

    let mut pull_5 = 0;
    let mut sum_5 = 0;
    let mut count_5 = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for warp in &warps {
        pull_4 += 1;
        pull_5 += 1;

        match warp.rarity.unwrap() {
            4 => {
                count_4 += 1;
                sum_4 += pull_4;
                pull_4 = 0;
            }
            5 => {
                count_5 += 1;
                sum_5 += pull_5;
                pull_5 = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [23000, 23002, 23003, 23004, 23005, 23012, 23013]
                        .contains(&warp.light_cone.unwrap())
                    {
                        win_streak = 0;

                        loss_streak += 1;
                        max_loss_streak = max_loss_streak.max(loss_streak);

                        guarantee = true;
                    } else {
                        sum_win += 1;

                        loss_streak = 0;

                        win_streak += 1;
                        max_win_streak = max_win_streak.max(win_streak);
                    }
                }
            }
            _ => {}
        }
    }

    let win_streak = max_win_streak;
    let loss_streak = max_loss_streak;

    let luck_4 = sum_4 as f64 / count_4 as f64;
    let luck_5 = sum_5 as f64 / count_5 as f64;
    let win_rate = sum_win as f64 / count_win as f64;

    let stat = database::warps_stats::lc::DbWarpsStatLc {
        uid,
        luck_4,
        luck_5,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::warps_stats::lc::set(&stat, pool).await?;

    Ok(())
}
