use std::time::{Duration, Instant};

use actix_web::rt::{self, time};
use sqlx::PgPool;

pub async fn achievements_percent(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = stardb_database::update_achievements_percent(&pool).await {
                log::error!(
                    "Achievements Percent update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                log::info!(
                    "Achievements Percent update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}
