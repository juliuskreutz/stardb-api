use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::Display;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps/{uid}")),
    paths(get_warps),
    components(schemas(GachaType, Warp))
)]
struct ApiDoc;

#[derive(Display, Deserialize, ToSchema)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum GachaType {
    Standard,
    Departure,
    Special,
    Lc,
}

#[derive(Serialize, ToSchema)]
struct Warp {
    r#type: WarpType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    timestamp: NaiveDateTime,
}

impl From<database::DbWarp> for Warp {
    fn from(warp: database::DbWarp) -> Self {
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

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

#[derive(Deserialize, IntoParams)]
struct WarpParams {
    gacha_type: GachaType,
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
    params(LanguageParams, WarpParams),
    responses(
        (status = 200, description = "[Warp]", body = Vec<Warp>),
    )
)]
#[get("/api/warps/{uid}")]
async fn get_warps(
    uid: web::Path<i64>,
    language_params: web::Query<LanguageParams>,
    warp_params: web::Query<WarpParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let warps: Vec<_> = database::get_warps_by_uid_and_gacha_type(
        *uid,
        &warp_params.gacha_type.to_string(),
        &language_params.lang.to_string(),
        &pool,
    )
    .await?
    .into_iter()
    .map(Warp::from)
    .collect();

    Ok(HttpResponse::Ok().json(warps))
}
