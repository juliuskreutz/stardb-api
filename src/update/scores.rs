use std::time::{Duration, Instant};

use actix_web::rt::{self, Runtime};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{mihomo, Language};

#[derive(Serialize, Deserialize)]
struct Score {
    uid: i32,
}

pub async fn spawn(pool: PgPool) {
    {
        let pool = pool.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            let handle = rt.spawn(async move {
                let mut interval = rt::time::interval(Duration::from_secs(60 * 10));

                loop {
                    interval.tick().await;

                    let start = Instant::now();

                    if let Err(e) = update_top_100(pool.clone()).await {
                        error!(
                            "Scores top 100 update failed with {e} in {}s",
                            start.elapsed().as_secs_f64()
                        );
                    } else {
                        info!(
                            "Scores top 100 update succeeded in {}s",
                            start.elapsed().as_secs_f64()
                        );
                    }
                }
            });

            rt.block_on(handle).unwrap();
        });
    }

    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let handle = rt.spawn(async move {
            let mut interval = rt::time::interval(Duration::from_secs(60 * 60 * 24));

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update_lower_100(pool.clone()).await {
                    error!(
                        "Scores lower 100 update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Scores lower 100 update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });

        rt.block_on(handle).unwrap();
    });
}

async fn update_top_100(pool: PgPool) -> Result<()> {
    let scores: Vec<Score> =
        reqwest::get("http://localhost:8000/api/scores/achievements?limit=100")
            .await?
            .json()
            .await?;

    update(scores, pool).await?;

    Ok(())
}

async fn update_lower_100(pool: PgPool) -> Result<()> {
    let scores: Vec<Score> =
        reqwest::get("http://localhost:8000/api/scores/achievements?offset=100")
            .await?
            .json()
            .await?;

    update(scores, pool).await?;

    Ok(())
}

async fn update(scores: Vec<Score>, pool: PgPool) -> Result<()> {
    for score in scores {
        loop {
            rt::time::sleep(std::time::Duration::from_secs(5)).await;

            if mihomo::update_and_get(score.uid, Language::En, &pool)
                .await
                .is_ok()
            {
                break;
            }
        }
    }

    Ok(())
}
