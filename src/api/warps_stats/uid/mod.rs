use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps_stats/{uid}")),
    paths(get_warps_stats),
    components(schemas())
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warps_stats);
}

#[derive(Serialize, ToSchema)]
struct WarpStats {}

#[utoipa::path(
    tag = "warps_stats/{uid}",
    get,
    path = "/api/warps_stats/{uid}",
    responses(
        (status = 200, description = "WarpStats", body = WarpStats),
    )
)]
#[get("/api/warps_stats/{uid}")]
async fn get_warps_stats(
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
