mod achievements;
mod uids;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(uids::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(uids::configure);
}
