mod signal_tracker;

use actix_web::web;
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(tags((name = "zzz/pages")))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(signal_tracker::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(signal_tracker::configure);
}
