use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Score {
    uid: i32,
}

pub async fn scores() {
    rt::spawn(async {
        let mut interval = rt::time::interval(Duration::from_secs(60 * 10));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update_top_100().await {
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

    rt::spawn(async {
        let mut interval = rt::time::interval(Duration::from_secs(60 * 60 * 24));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update_lower_100().await {
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
}

async fn update_top_100() -> Result<()> {
    let scores: Vec<Score> =
        reqwest::get("http://localhost:8000/api/scores/achievements?limit=100")
            .await?
            .json()
            .await?;

    update(scores).await?;

    Ok(())
}

async fn update_lower_100() -> Result<()> {
    let scores: Vec<Score> =
        reqwest::get("http://localhost:8000/api/scores/achievements?offset=100")
            .await?
            .json()
            .await?;

    update(scores).await?;

    Ok(())
}

async fn update(scores: Vec<Score>) -> Result<()> {
    let client = Arc::new(Client::new());

    for score in scores {
        loop {
            rt::time::sleep(std::time::Duration::from_secs(5)).await;

            if client
                .put(&format!("http://localhost:8000/api/mihomo/{}", score.uid))
                .send()
                .await
                .is_ok()
            {
                break;
            }
        }
    }

    Ok(())
}
