mod delete_all_signals;
mod delete_all_warps;
mod delete_all_wishes;
mod delete_unofficial_signals;
mod delete_unofficial_warps;
mod delete_unofficial_wishes;

use actix_web::web;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = delete_all_signals::openapi();
    openapi.merge(delete_all_warps::openapi());
    openapi.merge(delete_all_wishes::openapi());
    openapi.merge(delete_unofficial_signals::openapi());
    openapi.merge(delete_unofficial_warps::openapi());
    openapi.merge(delete_unofficial_wishes::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(delete_all_signals::configure)
        .configure(delete_all_warps::configure)
        .configure(delete_all_wishes::configure)
        .configure(delete_unofficial_signals::configure)
        .configure(delete_unofficial_warps::configure)
        .configure(delete_unofficial_wishes::configure);
}
