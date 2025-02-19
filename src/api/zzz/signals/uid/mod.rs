use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/signals/{uid}")),
    paths(get_zzz_signals),
    components(schemas(Signals, Signal, SignalType))
)]
struct ApiDoc;

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Signals {
    standard: Vec<Signal>,
    character: Vec<Signal>,
    w_engine: Vec<Signal>,
    bangboo: Vec<Signal>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Signal {
    r#type: SignalType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    timestamp: DateTime<Utc>,
}

impl From<database::zzz::signals::DbSignal> for Signal {
    fn from(signal: database::zzz::signals::DbSignal) -> Self {
        let r#type = if signal.character.is_some() {
            SignalType::Character
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
        }
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum SignalType {
    Character,
    WEngine,
    Bangboo,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_signals);
}

#[utoipa::path(
    tag = "zzz/signals/{uid}",
    get,
    path = "/api/zzz/signals/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Signals", body = Signals),
    )
)]
#[get("/api/zzz/signals/{uid}")]
async fn get_zzz_signals(
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

    let standard = database::zzz::signals::standard::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect();
    let character = database::zzz::signals::special::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect();
    let w_engine = database::zzz::signals::w_engine::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect();
    let bangboo = database::zzz::signals::bangboo::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect();

    let signals = Signals {
        standard,
        character,
        w_engine,
        bangboo,
    };

    Ok(HttpResponse::Ok().json(signals))
}
