use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps/{uid}")),
    paths(get_warps),
    components(schemas(Warp, GachaType))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Warp {
    name: String,
    timestamp: NaiveDateTime,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum GachaType {
    Standard,
    Departure,
    Special,
    Lc,
}

#[derive(Deserialize, IntoParams)]
struct WarpParams {
    gacha_type: GachaType,
}

impl From<database::DbWarpStandardCharacter> for Warp {
    fn from(warp_character: database::DbWarpStandardCharacter) -> Self {
        Self {
            name: warp_character.name,
            timestamp: warp_character.timestamp,
        }
    }
}

impl From<database::DbWarpStandardLightCone> for Warp {
    fn from(warp_light_cone: database::DbWarpStandardLightCone) -> Self {
        Self {
            name: warp_light_cone.name,
            timestamp: warp_light_cone.timestamp,
        }
    }
}

impl From<database::DbWarpDepartureCharacter> for Warp {
    fn from(warp_character: database::DbWarpDepartureCharacter) -> Self {
        Self {
            name: warp_character.name,
            timestamp: warp_character.timestamp,
        }
    }
}

impl From<database::DbWarpDepartureLightCone> for Warp {
    fn from(warp_light_cone: database::DbWarpDepartureLightCone) -> Self {
        Self {
            name: warp_light_cone.name,
            timestamp: warp_light_cone.timestamp,
        }
    }
}

impl From<database::DbWarpSpecialCharacter> for Warp {
    fn from(warp_character: database::DbWarpSpecialCharacter) -> Self {
        Self {
            name: warp_character.name,
            timestamp: warp_character.timestamp,
        }
    }
}

impl From<database::DbWarpSpecialLightCone> for Warp {
    fn from(warp_light_cone: database::DbWarpSpecialLightCone) -> Self {
        Self {
            name: warp_light_cone.name,
            timestamp: warp_light_cone.timestamp,
        }
    }
}

impl From<database::DbWarpLcCharacter> for Warp {
    fn from(warp_character: database::DbWarpLcCharacter) -> Self {
        Self {
            name: warp_character.name,
            timestamp: warp_character.timestamp,
        }
    }
}

impl From<database::DbWarpLcLightCone> for Warp {
    fn from(warp_light_cone: database::DbWarpLcLightCone) -> Self {
        Self {
            name: warp_light_cone.name,
            timestamp: warp_light_cone.timestamp,
        }
    }
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
        (status = 200, description = "[WarpSpecial]", body = Vec<WarpSpecial>),
    )
)]
#[get("/api/warps/{uid}")]
async fn get_warps(
    uid: web::Path<i64>,
    language_params: web::Query<LanguageParams>,
    warp_params: web::Query<WarpParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let (warp_characters, warp_light_cones) = match warp_params.gacha_type {
        GachaType::Standard => {
            let warp_standard_characters = database::get_warp_standard_characters_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            let warp_standard_light_cones = database::get_warp_standard_light_cones_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            (warp_standard_characters, warp_standard_light_cones)
        }
        GachaType::Departure => {
            let warp_departure_characters = database::get_warp_departure_characters_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            let warp_departure_light_cones = database::get_warp_departure_light_cones_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            (warp_departure_characters, warp_departure_light_cones)
        }
        GachaType::Special => {
            let warp_special_characters = database::get_warp_special_characters_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            let warp_special_light_cones = database::get_warp_special_light_cones_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            (warp_special_characters, warp_special_light_cones)
        }
        GachaType::Lc => {
            let warp_lc_characters = database::get_warp_lc_characters_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            let warp_lc_light_cones = database::get_warp_lc_light_cones_by_uid(
                *uid,
                &language_params.lang.to_string(),
                &pool,
            )
            .await?
            .into_iter()
            .map(Warp::from)
            .collect::<Vec<_>>();

            (warp_lc_characters, warp_lc_light_cones)
        }
    };

    let mut warp_characters = warp_characters.into_iter().peekable();
    let mut warp_light_cones = warp_light_cones.into_iter().peekable();

    let mut warps = Vec::new();

    loop {
        let n = match (warp_characters.peek(), warp_light_cones.peek()) {
            (Some(l), Some(r)) => {
                if l.timestamp < r.timestamp {
                    -1
                } else {
                    1
                }
            }
            (Some(_), None) => -1,
            (None, Some(_)) => 1,
            (None, None) => 0,
        };

        match n {
            -1 => warps.push(warp_characters.next().unwrap()),
            1 => warps.push(warp_light_cones.next().unwrap()),
            _ => break,
        }
    }

    Ok(HttpResponse::Ok().json(warps))
}
