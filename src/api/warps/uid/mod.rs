//! Localized HSR pull history, preserving the verified-owner privacy policy.

use crate::gacha::imports::PullItem as StoredItem;
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
    tags((name = "warps/{uid}")),
    paths(get_warps),
    components(schemas(Warps, Warp, WarpType))
)]
struct ApiDoc;

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Warps {
    departure: Vec<Warp>,
    standard: Vec<Warp>,
    character: Vec<Warp>,
    light_cone: Vec<Warp>,
    collab: Vec<Warp>,
    collab_lc: Vec<Warp>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Warp {
    r#type: WarpType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    timestamp: DateTime<Utc>,
}

impl TryFrom<database::warps::DbWarp> for Warp {
    type Error = anyhow::Error;
    /// Converts a validated pull to display fields, requiring a localized name.
    /// Missing labels return an error; item IDs and kinds come from the typed identity.
    fn try_from(warp: database::warps::DbWarp) -> anyhow::Result<Self> {
        let r#type = if matches!(warp.item, StoredItem::Character(_)) {
            WarpType::Character
        } else {
            WarpType::LightCone
        };

        Ok(Self {
            r#type,
            id: warp.id.to_string(),
            name: warp
                .name
                .ok_or_else(|| anyhow::anyhow!("missing localized pull name"))?,
            rarity: warp.rarity,
            item_id: warp.item.id(),
            timestamp: warp.timestamp,
        })
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

/// Returns the OpenAPI fragment for this module’s routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module’s HTTP routes with the application.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warps);
}

#[utoipa::path(
    tag = "warps/{uid}",
    get,
    path = "/api/warps/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Warps", body = Warps),
    )
)]
#[get("/api/warps/{uid}")]
/// Returns localized HSR histories, rejecting private UIDs without a verified
/// owner connection. Administrators do not bypass this endpoint’s existing policy.
/// Invalid stored rows or missing display names propagate as errors.
async fn get_warps(
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

    let departure = database::warps::departure::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let standard = database::warps::standard::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let character = database::warps::special::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let light_cone = database::warps::lc::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let collab = database::warps::collab::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let collab_lc = database::warps::collab_lc::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::try_from)
        .collect::<anyhow::Result<Vec<_>>>()?;

    let warps = Warps {
        departure,
        standard,
        character,
        light_cone,
        collab,
        collab_lc,
    };

    Ok(HttpResponse::Ok().json(warps))
}
