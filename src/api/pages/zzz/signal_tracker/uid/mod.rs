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
    pull_4: usize,
    pull_5: usize,
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
            pull_4: 0,
            pull_5: 0,
        }
    }
}

#[derive(Serialize)]
struct SignalTracker {
    standard: Signals,
    special: Signals,
    w_engine: Signals,
    bangboo: Signals,
}

#[derive(Default, Serialize)]
struct Signals {
    signals: Vec<Signal>,
    probability_4: f64,
    probability_5: f64,
    pull_4: usize,
    pull_5: usize,
    max_pull_4: usize,
    max_pull_5: usize,
    count: usize,
    stats: Stats,
}

#[derive(Default, Serialize)]
struct Stats {
    users: i32,
    count_percentile: f64,
    luck_4: f64,
    luck_4_percentile: f64,
    luck_5: f64,
    luck_5_percentile: f64,
    win_stats: Option<WinStats>,
}

#[derive(Serialize)]
struct WinStats {
    win_rate: f64,
    win_streak: i32,
    loss_streak: i32,
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

    let language = language_params.lang;

    // Standard
    let mut standard = Signals::default();
    let mut standard_pull = 0;
    let mut standard_pull_a = 0;
    let mut standard_pull_s = 0;

    for signal in database::zzz::signals::standard::get_by_uid(uid, language, &pool).await? {
        let mut signal: Signal = signal.into();

        standard_pull += 1;
        standard_pull_a += 1;
        standard_pull_s += 1;

        signal.pull = standard_pull;
        signal.pull_4 = standard_pull_a;
        signal.pull_5 = standard_pull_s;

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

    standard.pull_4 = standard_pull_a;
    standard.max_pull_4 = 10;
    standard.probability_4 = if standard_pull_a < 9 { 9.4 } else { 100.0 };

    standard.pull_5 = standard_pull_s;
    standard.max_pull_5 = 90;
    standard.probability_5 = if standard_pull_s < 89 {
        0.6 + 6.0 * standard_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    standard.count = standard.signals.len();
    // Standard

    // Special
    let mut special = Signals::default();
    let mut special_pull = 0;
    let mut special_pull_a = 0;
    let mut special_pull_s = 0;

    for signal in database::zzz::signals::special::get_by_uid(uid, language, &pool).await? {
        let mut signal: Signal = signal.into();

        special_pull += 1;
        special_pull_a += 1;
        special_pull_s += 1;

        signal.pull = special_pull;
        signal.pull_4 = special_pull_a;
        signal.pull_5 = special_pull_s;

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

    special.pull_4 = special_pull_a;
    special.max_pull_4 = 10;
    special.probability_4 = if special_pull_a < 9 { 9.4 } else { 100.0 };

    special.pull_5 = special_pull_s;
    special.max_pull_5 = 90;
    special.probability_5 = if special_pull_s < 89 {
        0.6 + 6.0 * special_pull_s.saturating_sub(72) as f64
    } else {
        100.0
    };

    special.count = special.signals.len();
    // Special

    // WEngine
    let mut w_engine = Signals::default();
    let mut w_engine_pull = 0;
    let mut w_engine_pull_a = 0;
    let mut w_engine_pull_s = 0;

    for signal in database::zzz::signals::w_engine::get_by_uid(uid, language, &pool).await? {
        let mut signal: Signal = signal.into();

        w_engine_pull += 1;
        w_engine_pull_a += 1;
        w_engine_pull_s += 1;

        signal.pull = w_engine_pull;
        signal.pull_4 = w_engine_pull_a;
        signal.pull_5 = w_engine_pull_s;

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

    w_engine.pull_4 = w_engine_pull_a;
    w_engine.max_pull_4 = 10;
    w_engine.probability_4 = if w_engine_pull_a < 9 { 15.0 } else { 100.0 };

    w_engine.pull_5 = w_engine_pull_s;
    w_engine.max_pull_5 = 80;
    w_engine.probability_5 = if w_engine_pull_s < 79 {
        1.0 + 7.0 * w_engine_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    w_engine.count = w_engine.signals.len();
    // WEngine

    // Bangboo
    let mut bangboo = Signals::default();
    let mut bangboo_pull = 0;
    let mut bangboo_pull_a = 0;
    let mut bangboo_pull_s = 0;

    for signal in database::zzz::signals::bangboo::get_by_uid(uid, language, &pool).await? {
        let mut signal: Signal = signal.into();

        bangboo_pull += 1;
        bangboo_pull_a += 1;
        bangboo_pull_s += 1;

        signal.pull = bangboo_pull;
        signal.pull_4 = bangboo_pull_a;
        signal.pull_5 = bangboo_pull_s;

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

    bangboo.pull_4 = bangboo_pull_a;
    bangboo.max_pull_4 = 10;
    bangboo.probability_4 = if bangboo_pull_a < 9 { 15.0 } else { 100.0 };

    bangboo.pull_5 = bangboo_pull_s;
    bangboo.max_pull_5 = 80;
    bangboo.probability_5 = if bangboo_pull_s < 79 {
        1.0 + 7.0 * bangboo_pull_s.saturating_sub(64) as f64
    } else {
        100.0
    };

    bangboo.count = bangboo.signals.len();
    // Bangboo

    if let Some(stats) = database::zzz::signals_stats::standard::get_by_uid(uid, &pool).await? {
        let users = database::zzz::signals_stats::standard::count(&pool).await? as i32;

        standard.stats = Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_a,
            luck_4_percentile: stats.luck_a_percentile,
            luck_5: stats.luck_s,
            luck_5_percentile: stats.luck_s_percentile,
            win_stats: None,
        };
    }

    if let Some(stats) = database::zzz::signals_stats::special::get_by_uid(uid, &pool).await? {
        let users = database::zzz::signals_stats::special::count(&pool).await? as i32;

        let win_stats = WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        };

        special.stats = Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_a,
            luck_4_percentile: stats.luck_a_percentile,
            luck_5: stats.luck_s,
            luck_5_percentile: stats.luck_s_percentile,
            win_stats: Some(win_stats),
        };
    }

    if let Some(stats) = database::zzz::signals_stats::w_engine::get_by_uid(uid, &pool).await? {
        let users = database::zzz::signals_stats::w_engine::count(&pool).await? as i32;

        let win_stats = WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        };

        w_engine.stats = Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_a,
            luck_4_percentile: stats.luck_a_percentile,
            luck_5: stats.luck_s,
            luck_5_percentile: stats.luck_s_percentile,
            win_stats: Some(win_stats),
        };
    }

    if let Some(stats) = database::zzz::signals_stats::bangboo::get_by_uid(uid, &pool).await? {
        let users = database::zzz::signals_stats::bangboo::count(&pool).await? as i32;

        bangboo.stats = Stats {
            users,
            count_percentile: stats.count_percentile,
            luck_4: stats.luck_a,
            luck_4_percentile: stats.luck_a_percentile,
            luck_5: stats.luck_s,
            luck_5_percentile: stats.luck_s_percentile,
            win_stats: None,
        };
    }

    let signal_tracker = SignalTracker {
        standard,
        special,
        w_engine,
        bangboo,
    };

    Ok(HttpResponse::Ok().json(signal_tracker))
}
