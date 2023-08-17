mod achievement_tracker;

use actix_web::web;
use sqlx::PgPool;

pub fn cache_achievement_tracker(
    pool: PgPool,
) -> web::Data<achievement_tracker::AchievementTrackerCache> {
    achievement_tracker::cache(pool)
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievement_tracker::configure);
}
