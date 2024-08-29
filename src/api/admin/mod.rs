mod delete_unofficial_warps;

use actix_web::web;

pub fn openapi() -> utoipa::openapi::OpenApi {
    delete_unofficial_warps::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(delete_unofficial_warps::configure);
}
