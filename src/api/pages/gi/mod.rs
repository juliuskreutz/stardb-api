mod achievement_tracker;
mod wish_tracker;

use crate::app_config::AppConfig;
use actix_web::web;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(tags((name = "gi/pages")))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievement_tracker::openapi());
    openapi.merge(wish_tracker::openapi());
    openapi
}

pub fn configure(
    cfg: &mut web::ServiceConfig,
    pool: PgPool,
    app_config: web::Data<Arc<AppConfig>>,
) {
    cfg.configure(|sc| achievement_tracker::configure(sc, pool, app_config))
        .configure(wish_tracker::configure);
}
