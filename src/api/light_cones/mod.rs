mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

use super::LanguageParams;

#[derive(OpenApi)]
#[openapi(
    tags((name = "light-cones")),
    paths(get_light_cones),
    components(schemas(
        LightCone
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct LightCone {
    id: i32,
    name: String,
}

impl From<database::DbLightCone> for LightCone {
    fn from(db_light_cone: database::DbLightCone) -> Self {
        Self {
            id: db_light_cone.id,
            name: db_light_cone.name,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_light_cones).configure(id::configure);
}

#[utoipa::path(
    tag = "light-cones",
    get,
    path = "/api/light-cones",
    params(LanguageParams),
    responses(
        (status = 200, description = "[LightCone]", body = Vec<LightCone>),
    )
)]
#[get("/api/light-cones")]
async fn get_light_cones(
    language_param: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let light_cones: Vec<_> = database::get_light_cones(&language_param.lang.to_string(), &pool)
        .await?
        .into_iter()
        .map(LightCone::from)
        .collect();

    Ok(HttpResponse::Ok().json(light_cones))
}
