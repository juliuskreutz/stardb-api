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

    for score in scores {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

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

    // futures::stream::iter(scores)
    //     .map(|s| (client.clone(), s))
    //     .map(|(client, score)| async move {
    //         loop {
    //             if client
    //                 .put(&format!("http://localhost:8000/api/mihomo/{}", score.uid))
    //                 .send()
    //                 .await
    //                 .is_ok()
    //             {
    //                 break;
    //             }
    //         }
    //     })
    //     .buffer_unordered(16)
    //     .collect::<Vec<_>>()
    //     .await;

    Ok(())
}
