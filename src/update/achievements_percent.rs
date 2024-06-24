use std::time::{Duration, Instant};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn achievements_percent(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = rt::time::interval(Duration::from_secs(5 * 60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(pool.clone()).await {
                error!(
                    "Achievements Percent update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                info!(
                    "Achievements Percent update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: PgPool) -> Result<()> {
    database::achievements_percent::update(&pool).await?;

    Ok(())
}
