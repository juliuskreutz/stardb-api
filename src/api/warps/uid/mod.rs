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
        }
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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
        .map(Warp::from)
        .collect();
    let standard = database::warps::standard::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::from)
        .collect();
    let character = database::warps::special::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::from)
        .collect();
    let light_cone = database::warps::lc::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Warp::from)
        .collect();

    let warps = Warps {
        departure,
        standard,
        character,
        light_cone,
    };

    Ok(HttpResponse::Ok().json(warps))
}
