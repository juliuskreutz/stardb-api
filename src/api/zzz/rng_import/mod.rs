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
    let exclusive_rescreening_signals: Vec<Signal> = signals
        .get(ZzzGachaType::ExclusiveRescreening.id().to_string())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let w_engine_reverberation_signals: Vec<Signal> = signals
        .get(ZzzGachaType::WEngineReverberation.id().to_string())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    for (signals, gacha_type) in [
        (standard_signals, ZzzGachaType::Standard),
        (special_signals, ZzzGachaType::Special),
        (w_engine_signals, ZzzGachaType::WEngine),
        (bangboo_signals, ZzzGachaType::Bangboo),
        (exclusive_rescreening_signals, ZzzGachaType::ExclusiveRescreening),
        (w_engine_reverberation_signals, ZzzGachaType::WEngineReverberation),
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
            },
            ZzzGachaType::ExclusiveRescreening => {
                database::zzz::signals::exclusive_rescreening::set_all(&set_all, &pool).await?
            },
            ZzzGachaType::WEngineReverberation => {
                database::zzz::signals::w_engine_reverberation::set_all(&set_all, &pool).await?
            },
        }
    }

    Ok(HttpResponse::Ok().finish())
}
