mod achievement_tracker;
mod signal_tracker;

use actix_web::web;
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(tags((name = "zzz/pages")))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievement_tracker::openapi());
    openapi.merge(signal_tracker::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    cfg.configure(|sc| achievement_tracker::configure(sc, pool))
        .configure(signal_tracker::configure);
}
