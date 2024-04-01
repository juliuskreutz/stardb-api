pub mod achievement_tracker;
pub mod book_tracker;
mod community_tier_list;
mod leaderboard;
mod profiles;
mod warp_tracker;

use actix_web::web;
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(tags((name = "pages")))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievement_tracker::openapi());
    openapi.merge(book_tracker::openapi());
    openapi.merge(community_tier_list::openapi());
    openapi.merge(leaderboard::openapi());
    openapi.merge(profiles::openapi());
    openapi.merge(warp_tracker::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievement_tracker::configure)
        .configure(book_tracker::configure)
        .configure(community_tier_list::configure)
        .configure(leaderboard::configure)
        .configure(profiles::configure)
        .configure(warp_tracker::configure);
}

pub fn cache_book_tracker(pool: PgPool) -> web::Data<book_tracker::BookTrackerCache> {
    book_tracker::cache(pool)
}
