pub mod achievement_tracker;
mod community_tier_list;
mod leaderboard;

use actix_web::web;
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(tags((name = "pages")))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievement_tracker::openapi());
    openapi.merge(community_tier_list::openapi());
    openapi.merge(leaderboard::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievement_tracker::configure)
        .configure(community_tier_list::configure)
        .configure(leaderboard::configure);
}

pub fn cache_achievement_tracker(
    pool: PgPool,
) -> web::Data<achievement_tracker::AchievementTrackerCache> {
    achievement_tracker::cache(pool)
}
