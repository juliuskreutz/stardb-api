use std::{sync::Arc, time::Duration};

use actix_web::rt::{self, time};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::Result;

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
        let minutes = 10;

        let mut timer = time::interval(Duration::from_secs(60 * minutes));

        loop {
            timer.tick().await;

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
