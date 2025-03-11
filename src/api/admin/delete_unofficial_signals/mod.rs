mod uid;

use actix_web::web;

pub fn openapi() -> utoipa::openapi::OpenApi {
    uid::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(uid::configure);
}
