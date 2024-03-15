use std::time::{Duration, Instant};

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn achievements_percent(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
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

async fn update(pool: &PgPool) -> Result<()> {
    database::update_achievements_percent(pool).await?;

    Ok(())
}
