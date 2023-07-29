use actix_web::web;
use utoipa::OpenApi;

mod uid;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    uid::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(uid::configure);
}
