use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, ZzzGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/rng-signals-import")),
    paths(post_rng_signals_import),
    components(schemas(RngSignalsImportParams))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_rng_signals_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct RngSignalsImportParams {
    data: String,
    profile: i32,
}

#[derive(serde::Deserialize)]
struct Signal {
    uid: String,
    id: i32,
    timestamp: i64,
}

#[utoipa::path(
    tag = "zzz/rng-signals-import",
    post,
    path = "/api/zzz/rng-signals-import",
    request_body = RngSignalsImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not verified"),
    )
)]
#[post("/api/zzz/rng-signals-import")]
async fn post_rng_signals_import(
    session: Session,
    params: web::Json<RngSignalsImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let json: serde_json::Value = serde_json::from_str(&params.data)?;

    let profile = json["data"]["profiles"][&params.profile.to_string()].clone();

    let Some(uid) = profile["bindUid"].as_i64() else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let uid = uid as i32;

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin && database::zzz::uids::get_by_uid(uid, &pool).await.is_err() {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let allowed = admin
        || database::zzz::connections::get_by_username(&username, &pool)
            .await?
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.verified)
            .unwrap_or_default();

    if !allowed {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let signals = profile["stores"]["0"]["items"].clone();

    let standard_signals: Vec<Signal> = signals
        .get(ZzzGachaType::Standard.id().to_string())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let special_signals: Vec<Signal> = signals
        .get(ZzzGachaType::Special.id().to_string())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let w_engine_signals: Vec<Signal> = signals
        .get(ZzzGachaType::WEngine.id().to_string())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let bangboo_signals: Vec<Signal> = signals
        .get(ZzzGachaType::Bangboo.id().to_string())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    for (signals, gacha_type) in [
        (standard_signals, ZzzGachaType::Standard),
        (special_signals, ZzzGachaType::Special),
        (w_engine_signals, ZzzGachaType::WEngine),
        (bangboo_signals, ZzzGachaType::Bangboo),
    ] {
        let mut set_all = database::zzz::signals::SetAll::default();

        for signal in signals {
            let id = signal.uid.parse()?;

            let mut character = None;
            let mut w_engine = None;
            let mut bangboo = None;

            if signal.id >= 50000 {
                bangboo = Some(signal.id);
            } else if signal.id >= 12000 {
                w_engine = Some(signal.id);
            } else {
                character = Some(signal.id);
            }

            let timestamp = chrono::DateTime::from_timestamp(
                if signal.timestamp > 1_000_000_000_000 {
                    signal.timestamp / 1000
                } else {
                    signal.timestamp
                },
                0,
            )
            .unwrap();

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.w_engine.push(w_engine);
            set_all.bangboo.push(bangboo);
            set_all.timestamp.push(timestamp);
            set_all.official.push(false);
        }

        match gacha_type {
            ZzzGachaType::Standard => {
                database::zzz::signals::standard::set_all(&set_all, &pool).await?
            }
            ZzzGachaType::Special => {
                database::zzz::signals::special::set_all(&set_all, &pool).await?
            }
            ZzzGachaType::WEngine => {
                database::zzz::signals::w_engine::set_all(&set_all, &pool).await?
            }
            ZzzGachaType::Bangboo => {
                database::zzz::signals::bangboo::set_all(&set_all, &pool).await?
            }
        }
    }

    calculate_stats(uid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

async fn calculate_stats(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    calculate_stats_standard(uid, pool).await?;
    calculate_stats_special(uid, pool).await?;
    calculate_stats_w_engine(uid, pool).await?;
    calculate_stats_bangboo(uid, pool).await?;

    Ok(())
}

async fn calculate_stats_standard(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::standard::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut first_s_rank = true;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                if first_s_rank {
                    first_s_rank = false;
                    pull_s = 0;
                    continue;
                }

                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;
            }
            _ => {}
        }
    }

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };

    let stat = database::zzz::signals_stats::standard::DbSignalsStatStandard {
        uid,
        luck_a,
        luck_s,
    };
    database::zzz::signals_stats::standard::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_special(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::special::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [1021, 1041, 1101, 1141, 1181, 1211].contains(&signal.character.unwrap()) {
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

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    let stat = database::zzz::signals_stats::special::DbSignalsStatSpecial {
        uid,
        luck_a,
        luck_s,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::zzz::signals_stats::special::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_w_engine(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::w_engine::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    let mut guarantee = false;

    let mut sum_win = 0;
    let mut count_win = 0;

    let mut win_streak = 0;
    let mut max_win_streak = 0;

    let mut loss_streak = 0;
    let mut max_loss_streak = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;

                if guarantee {
                    guarantee = false;
                } else {
                    count_win += 1;

                    if [14102, 14104, 14110, 14114, 14118, 14121]
                        .contains(&signal.w_engine.unwrap())
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

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };
    let win_rate = if count_win != 0 {
        sum_win as f64 / count_win as f64
    } else {
        0.0
    };

    let stat = database::zzz::signals_stats::w_engine::DbSignalsStatWEngine {
        uid,
        luck_a,
        luck_s,
        win_rate,
        win_streak,
        loss_streak,
    };
    database::zzz::signals_stats::w_engine::set(&stat, pool).await?;

    Ok(())
}

async fn calculate_stats_bangboo(uid: i32, pool: &PgPool) -> anyhow::Result<()> {
    let signals = database::zzz::signals::bangboo::get_infos_by_uid(uid, pool).await?;

    let mut pull_a = 0;
    let mut sum_a = 0;
    let mut count_a = 0;

    let mut pull_s = 0;
    let mut sum_s = 0;
    let mut count_s = 0;

    for signal in &signals {
        pull_a += 1;
        pull_s += 1;

        match signal.rarity.unwrap() {
            3 => {
                count_a += 1;
                sum_a += pull_a;
                pull_a = 0;
            }
            4 => {
                count_s += 1;
                sum_s += pull_s;
                pull_s = 0;
            }
            _ => {}
        }
    }

    let luck_a = if count_a != 0 {
        sum_a as f64 / count_a as f64
    } else {
        0.0
    };
    let luck_s = if count_s != 0 {
        sum_s as f64 / count_s as f64
    } else {
        0.0
    };

    let stat = database::zzz::signals_stats::bangboo::DbSignalsStatBangboo {
        uid,
        luck_a,
        luck_s,
    };
    database::zzz::signals_stats::bangboo::set(&stat, pool).await?;

    Ok(())
}
