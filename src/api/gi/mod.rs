mod wishes_import;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(wishes_import::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(wishes_import::configure);
}
