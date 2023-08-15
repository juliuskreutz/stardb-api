use std::sync::Arc;

use actix_web::rt;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use anyhow::Result;

#[derive(Deserialize)]
struct Scores {
    scores: Vec<Score>,
}

#[derive(Serialize, Deserialize)]
struct Score {
    uid: i64,
}

pub async fn scores() {
    rt::spawn(async {
        loop {
            let _ = update().await;
        }
    });
}

async fn update() -> Result<()> {
    let client = Arc::new(Client::new());

    let scores: Scores = client
        .get("http://localhost:8000/api/scores/achievements")
        .send()
        .await?
        .json()
        .await?;

    futures::stream::iter(scores.scores)
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
