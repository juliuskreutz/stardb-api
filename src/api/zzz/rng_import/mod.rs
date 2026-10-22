use crate::gacha::imports::{NormalizedPull, PullItem, PullPool, PullProvenance};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, ZzzGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/rng-signals-import")),
    paths(post_rng_signals_import),
    components(schemas(RngSignalsImportParams))
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
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
/// Imports the selected RNG profile after admin or verified ZZZ authorization.
/// Malformed profile data or pull batches return 400; unauthorized claims return 403.
/// All retained pulls and recalculated stats commit together as unofficial data.
async fn post_rng_signals_import(
    session: Session,
    params: web::Json<RngSignalsImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&params.data) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let profile = json["data"]["profiles"][&params.profile.to_string()].clone();

    let Some(uid) = profile["bindUid"].as_i64() else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Ok(uid) = i32::try_from(uid) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

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

    let Ok(normalized_pulls) = parse_signals(uid, &profile["stores"]["0"]["items"]) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(batch) =
        crate::gacha::imports::ImportBatch::new(normalized_pulls, PullProvenance::Unofficial)
    else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    crate::gacha::imports::persist_batch_in_transaction(&batch, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

/// Parses RNG's pool-keyed items object into unofficial pulls without writing data.
/// An empty object is valid; missing items, malformed IDs or out-of-range timestamps
/// return errors for the endpoint to map to 400.
fn parse_signals(uid: i32, items: &serde_json::Value) -> anyhow::Result<Vec<NormalizedPull>> {
    let items = items
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("missing signal items"))?;
    let mut pulls = Vec::new();
    for gacha_type in ZzzGachaType::iter() {
        let Some(values) = items.get(&gacha_type.old_id().to_string()) else {
            continue;
        };
        for signal in serde_json::from_value::<Vec<Signal>>(values.clone())? {
            let item = if signal.id >= 50000 {
                PullItem::Bangboo(signal.id)
            } else if signal.id >= 12000 {
                PullItem::WEngine(signal.id)
            } else {
                PullItem::Character(signal.id)
            };
            let seconds = if signal.timestamp > 1_000_000_000_000 {
                signal.timestamp / 1000
            } else {
                signal.timestamp
            };
            let timestamp = chrono::DateTime::from_timestamp(seconds, 0)
                .ok_or_else(|| anyhow::anyhow!("invalid signal timestamp"))?;
            pulls.push(NormalizedPull {
                uid,
                id: signal.uid.parse()?,
                pool: PullPool::Zzz(gacha_type),
                item,
                timestamp,
                provenance: PullProvenance::Unofficial,
            });
        }
    }
    Ok(pulls)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_export_is_valid_but_missing_or_malformed_items_are_not() {
        assert!(parse_signals(1, &serde_json::json!({})).unwrap().is_empty());
        assert!(parse_signals(1, &serde_json::Value::Null).is_err());
        let key = ZzzGachaType::Standard.old_id().to_string();
        for value in [
            serde_json::json!("bad"),
            serde_json::json!([{"uid":"bad","id":1,"timestamp":0}]),
            serde_json::json!([{"uid":"1","id":1,"timestamp":i64::MAX}]),
        ] {
            assert!(parse_signals(1, &serde_json::json!({key.clone():value})).is_err());
        }
    }
}
