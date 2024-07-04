mod signals_import;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(signals_import::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(signals_import::configure);
}
