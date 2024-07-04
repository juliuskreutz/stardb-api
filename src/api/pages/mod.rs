mod achievement_tracker;
mod zzz;
//mod book_tracker;
//mod community_tier_list;
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
    //openapi.merge(book_tracker::openapi());
    //openapi.merge(community_tier_list::openapi());
    openapi.merge(leaderboard::openapi());
    openapi.merge(profiles::openapi());
    openapi.merge(warp_tracker::openapi());
    openapi.merge(zzz::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    cfg.configure(|sc| achievement_tracker::configure(sc, pool.clone()))
        //.configure(|sc| book_tracker::configure(sc, pool.clone()))
        //.configure(community_tier_list::configure)
        .configure(leaderboard::configure)
        .configure(profiles::configure)
        .configure(warp_tracker::configure)
        .configure(zzz::configure);
}
