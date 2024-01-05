use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Score {
    uid: i64,
}

pub async fn scores() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60 * 24));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update().await {
                log::error!(
                    "Scores update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                log::info!(
                    "Scores update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update() -> Result<()> {
    let client = Arc::new(Client::new());

    let scores: Vec<Score> = client
        .get("http://localhost:8000/api/scores/achievements")
        .send()
        .await?
        .json()
        .await?;

    for score in scores {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

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
