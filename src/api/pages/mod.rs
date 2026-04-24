mod achievement_tracker;
mod gi;
mod leaderboard;
mod profiles;
mod warp_tracker;
mod zzz;

use crate::app_config::AppConfig;
use actix_web::web;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(tags((name = "pages")))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievement_tracker::openapi());
    openapi.merge(leaderboard::openapi());
    openapi.merge(profiles::openapi());
    openapi.merge(warp_tracker::openapi());
    openapi.merge(gi::openapi());
    openapi.merge(zzz::openapi());
    openapi
}

pub fn configure(
    cfg: &mut web::ServiceConfig,
    pool: PgPool,
    app_config: web::Data<Arc<AppConfig>>,
) {
    cfg.configure(|sc| achievement_tracker::configure(sc, pool.clone(), app_config.clone()))
        .configure(leaderboard::configure)
        .configure(profiles::configure)
        .configure(warp_tracker::configure)
        .configure(|sc| gi::configure(sc, pool.clone(), app_config.clone()))
        .configure(|sc| zzz::configure(sc, pool.clone(), app_config.clone()));
}
