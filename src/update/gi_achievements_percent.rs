use std::time::{Duration, Instant};

use actix_web::rt::{self, Runtime};
use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn spawn(pool: PgPool) {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let handle = rt.spawn(async move {
            let mut interval = rt::time::interval(Duration::from_secs(60 * 60));

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update(pool.clone()).await {
                    error!(
                        "Gi Achievements Percent update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Gi Achievements Percent update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

async fn update(pool: PgPool) -> Result<()> {
    database::gi::achievements_percent::update(&pool).await?;

    Ok(())
}
