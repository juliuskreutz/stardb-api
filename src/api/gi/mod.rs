mod achievements;
mod banners;
mod paimon_wishes_import;
mod uigf_wishes_import;
mod wishes;
mod wishes_import;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(banners::openapi());
    openapi.merge(paimon_wishes_import::openapi());
    openapi.merge(uigf_wishes_import::openapi());
    openapi.merge(wishes::openapi());
    openapi.merge(wishes_import::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(banners::configure)
        .configure(paimon_wishes_import::configure)
        .configure(uigf_wishes_import::configure)
        .configure(wishes::configure)
        .configure(wishes_import::configure);
}
