use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams},
    database, ZzzGachaType,
};

#[derive(OpenApi)]
#[openapi(paths(get_signal_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_signal_tracker);
}

#[derive(Serialize)]
struct Signal {
    r#type: SignalType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    pull: usize,
    pull_a: usize,
    pull_s: usize,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum SignalType {
    Agent,
    WEngine,
    Bangboo,
}

impl From<database::zzz::signals::DbSignal> for Signal {
    fn from(signal: database::zzz::signals::DbSignal) -> Self {
        let r#type = if signal.character.is_some() {
            SignalType::Agent
        } else if signal.w_engine.is_some() {
            SignalType::WEngine
        } else {
            SignalType::Bangboo
        };

        Self {
            r#type,
            id: signal.id.to_string(),
            name: signal.name.unwrap(),
            rarity: signal.rarity.unwrap(),
            item_id: signal
                .character
                .or(signal.w_engine)
                .or(signal.bangboo)
                .unwrap(),
            timestamp: signal.timestamp,
            pull: 0,
            pull_a: 0,
            pull_s: 0,
        }
    }
}

#[derive(Serialize)]
struct SignalTracker {
    standard: Signals,
    special: Signals,
    w_engine: Signals,
    bangboo: Signals,
    count: usize,
    polychromes: usize,
}

#[derive(Default, Serialize)]
struct Signals {
    signals: Vec<Signal>,
    probability_a: f64,
    probability_s: f64,
    pull_a: usize,
    pull_s: usize,
    max_pull_a: usize,
    max_pull_s: usize,
    count: usize,
    polychromes: usize,
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/zzz/signal-tracker/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "SignalTracker"),
    )
)]
#[get("/api/pages/zzz/signal-tracker/{uid}", guard = "private")]
async fn get_signal_tracker(
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::zzz::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if let Ok(connection) =
                database::zzz::connections::get_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let signals = database::zzz::signals::get_by_uid(uid, language_params.lang, &pool).await?;

    let mut standard = Signals::default();
    let mut special = Signals::default();
    let mut w_engine = Signals::default();
    let mut bangboo = Signals::default();

    let mut standard_pull = 0;
    let mut special_pull = 0;
    let mut w_engine_pull = 0;
    let mut bangboo_pull = 0;

    let mut standard_pull_a = 0;
    let mut special_pull_a = 0;
    let mut w_engine_pull_a = 0;
    let mut bangboo_pull_a = 0;

    let mut standard_pull_s = 0;
    let mut special_pull_s = 0;
    let mut w_engine_pull_s = 0;
    let mut bangboo_pull_s = 0;

    for signal in signals {
        let gacha_type = signal.gacha_type.parse()?;

        let mut signal: Signal = signal.into();

        match gacha_type {
            ZzzGachaType::Standard => {
                standard_pull += 1;
                standard_pull_a += 1;
                standard_pull_s += 1;

                signal.pull = standard_pull;
                signal.pull_a = standard_pull_a;
                signal.pull_s = standard_pull_s;

                match signal.rarity {
                    3 => standard_pull_a = 0,
                    4 => {
                        standard_pull_a = 0;
                        standard_pull_s = 0;
                    }
                    _ => {}
                }

                standard.signals.push(signal);
            }
            ZzzGachaType::Special => {
                special_pull += 1;
                special_pull_a += 1;
                special_pull_s += 1;

                signal.pull = special_pull;
                signal.pull_a = special_pull_a;
                signal.pull_s = special_pull_s;

                match signal.rarity {
                    3 => special_pull_a = 0,
                    4 => {
                        special_pull_a = 0;
                        special_pull_s = 0;
                    }
                    _ => {}
                }

                special.signals.push(signal);
            }
            ZzzGachaType::WEngine => {
                w_engine_pull += 1;
                w_engine_pull_a += 1;
                w_engine_pull_s += 1;

                signal.pull = w_engine_pull;
                signal.pull_a = w_engine_pull_a;
                signal.pull_s = w_engine_pull_s;

                match signal.rarity {
                    3 => w_engine_pull_a = 0,
                    4 => {
                        w_engine_pull_a = 0;
                        w_engine_pull_s = 0;
                    }
                    _ => {}
                }

                w_engine.signals.push(signal);
            }
            ZzzGachaType::Bangboo => {
                bangboo_pull += 1;
                bangboo_pull_a += 1;
                bangboo_pull_s += 1;

                signal.pull = bangboo_pull;
                signal.pull_a = bangboo_pull_a;
                signal.pull_s = bangboo_pull_s;

                match signal.rarity {
                    3 => bangboo_pull_a = 0,
                    4 => {
                        bangboo_pull_a = 0;
                        bangboo_pull_s = 0;
                    }
                    _ => {}
                }

                bangboo.signals.push(signal);
            }
        }
    }

    standard.pull_a = standard_pull_a;
    standard.max_pull_a = 10;
    standard.probability_a = if standard_pull_a < 10 { 9.4 } else { 100.0 };

    special.pull_a = special_pull_a;
    special.max_pull_a = 10;
    special.probability_a = if special_pull_a < 10 { 9.4 } else { 100.0 };

    w_engine.pull_a = w_engine_pull_a;
    w_engine.max_pull_a = 10;
    w_engine.probability_a = if w_engine_pull_a < 10 { 15.0 } else { 100.0 };

    bangboo.pull_a = bangboo_pull_a;
    bangboo.max_pull_a = 10;
    bangboo.probability_a = if bangboo_pull_a < 10 { 15.0 } else { 100.0 };

    standard.pull_s = standard_pull_s;
    standard.max_pull_s = 90;
    standard.probability_s = if standard_pull_s < 89 {
        0.6 + 6.0 * standard_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    special.pull_s = special_pull_s;
    special.max_pull_s = 90;
    special.probability_s = if special_pull_s < 89 {
        0.6 + 6.0 * special_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    w_engine.pull_s = w_engine_pull_s;
    w_engine.max_pull_s = 80;
    w_engine.probability_s = if w_engine_pull_s < 79 {
        1.0 + 7.0 * w_engine_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    bangboo.pull_s = bangboo_pull_s;
    bangboo.max_pull_s = 80;
    bangboo.probability_s = if bangboo_pull_s < 79 {
        1.0 + 7.0 * bangboo_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    standard.count = standard.signals.len();
    w_engine.count = w_engine.signals.len();
    special.count = special.signals.len();
    bangboo.count = bangboo.signals.len();

    standard.polychromes = standard.count * 160;
    w_engine.polychromes = w_engine.count * 160;
    special.polychromes = special.count * 160;
    bangboo.polychromes = bangboo.count * 160;

    let count = standard.count + w_engine.count + special.count + bangboo.count;
    let polychromes =
        standard.polychromes + w_engine.polychromes + special.polychromes + bangboo.polychromes;

    let signal_tracker = SignalTracker {
        standard,
        special,
        w_engine,
        bangboo,
        count,
        polychromes,
    };

    Ok(HttpResponse::Ok().json(signal_tracker))
}
