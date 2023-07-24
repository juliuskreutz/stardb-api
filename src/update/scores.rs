use actix_web::rt;
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
        loop {
            let _ = update_100().await;
        }
    });

    rt::spawn(async {
        loop {
            let _ = update_rest().await;
        }
    });
}

async fn update_100() -> Result<()> {
    let client = Client::new();

    let scores: Scores = client
        .get("http://localhost:8000/api/scores?limit=100")
        .send()
        .await?
        .json()
        .await?;

    for score in scores.scores {
        client
            .put(&format!("http://localhost:8000/api/scores/{}", score.uid))
            .send()
            .await?;
    }

    Ok(())
}

async fn update_rest() -> Result<()> {
    let client = Client::new();

    let scores: Scores = client
        .get("http://localhost:8000/api/scores?offset=100")
        .send()
        .await?
        .json()
        .await?;

    for score in scores.scores {
        client
            .put(&format!("http://localhost:8000/api/scores/{}", score.uid))
            .send()
            .await?;
    }

    Ok(())
}
