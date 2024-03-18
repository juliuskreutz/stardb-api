use std::time::{Duration, Instant};

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn warps_stats(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                error!(
                    "Warps stats update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                info!(
                    "Warps stats update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    database::update_warps_avg(pool).await
}
