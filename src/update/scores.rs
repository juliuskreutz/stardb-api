use std::{sync::Arc, time::Instant};

use actix_web::rt;
use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Score {
    uid: i64,
}

pub async fn scores() {
    rt::spawn(async {
        loop {
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

    futures::stream::iter(scores)
        .map(|s| (client.clone(), s))
        .map(|(client, score)| async move {
            loop {
                if client
                    .put(&format!("http://localhost:8000/api/mihomo/{}", score.uid))
                    .send()
                    .await
                    .is_ok()
                {
                    break;
                }
            }
        })
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;

    Ok(())
}
