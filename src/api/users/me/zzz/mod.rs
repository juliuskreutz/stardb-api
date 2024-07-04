mod uids;

use actix_web::web;

pub fn openapi() -> utoipa::openapi::OpenApi {
    uids::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(uids::configure);
}
