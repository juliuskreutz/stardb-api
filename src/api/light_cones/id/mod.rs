use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{light_cones::LightCone, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(tags((name = "light-cones/{id}")), paths(get_light_cone))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_light_cone);
}

#[utoipa::path(
    tag = "light-cones/{id}",
    get,
    path = "/api/light-cones/{id}",
    params(LanguageParams),
    responses(
        (status = 200, description = "LightCone", body = LightCone),
    )
)]
#[get("/api/light-cones/{id}")]
async fn get_light_cone(
    id: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let light_cone =
        LightCone::from(database::light_cones::get_by_id(*id, language_params.lang, &pool).await?);

    Ok(HttpResponse::Ok().json(light_cone))
}
