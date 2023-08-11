mod achievements;
mod characters;
mod free_jade_alert;
pub mod import;
mod mihomo;
mod scores;
mod series;
mod users;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(characters::openapi());
    openapi.merge(free_jade_alert::openapi());
    openapi.merge(import::openapi());
    openapi.merge(mihomo::openapi());
    openapi.merge(scores::openapi());
    openapi.merge(series::openapi());
    openapi.merge(users::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(characters::configure)
        .configure(free_jade_alert::configure)
        .configure(import::configure)
        .configure(mihomo::configure)
        .configure(scores::configure)
        .configure(series::configure)
        .configure(users::configure);
}
