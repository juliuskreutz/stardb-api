use crate::gacha::imports::PullItem as StoredItem;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{gacha_history_forbidden, ApiResult, LanguageParams},
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
    exclusive_rescreening: Vec<Signal>,
    w_engine_reverberation: Vec<Signal>,
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

impl TryFrom<database::zzz::signals::DbSignal> for Signal {
    type Error = anyhow::Error;
    fn try_from(signal: database::zzz::signals::DbSignal) -> anyhow::Result<Self> {
        let r#type = if matches!(signal.item, StoredItem::Character(_)) {
            SignalType::Character
        } else if matches!(signal.item, StoredItem::WEngine(_)) {
            SignalType::WEngine
        } else {
            SignalType::Bangboo
        };

        Ok(Self {
            r#type,
            id: signal.id.to_string(),
            name: signal
                .name
                .ok_or_else(|| anyhow::anyhow!("missing localized pull name"))?,
            rarity: signal.rarity,
            item_id: signal.item.id(),
            timestamp: signal.timestamp,
        })
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

    let is_private = database::zzz::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);
    let mut forbidden = is_private;

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            let is_admin = database::admins::exists(&username, &pool).await?;
            let has_verified_connection =
                database::zzz::connections::get_by_uid_and_username(uid, &username, &pool)
                    .await
                    .is_ok_and(|connection| connection.verified);
            forbidden = gacha_history_forbidden(is_admin, has_verified_connection);
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
        .collect::<anyhow::Result<Vec<_>>>()?;
    let character = database::zzz::signals::special::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let w_engine = database::zzz::signals::w_engine::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let bangboo = database::zzz::signals::bangboo::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Signal::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let exclusive_rescreening =
        database::zzz::signals::exclusive_rescreening::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(Signal::from)
            .collect::<anyhow::Result<Vec<_>>>()?;
    let w_engine_reverberation =
        database::zzz::signals::w_engine_reverberation::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(Signal::from)
            .collect::<anyhow::Result<Vec<_>>>()?;

    let signals = Signals {
        standard,
        character,
        w_engine,
        bangboo,
        exclusive_rescreening,
        w_engine_reverberation,
    };

    Ok(HttpResponse::Ok().json(signals))
}
