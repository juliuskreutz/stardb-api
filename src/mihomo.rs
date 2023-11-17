use std::fs::File;

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use anyhow::Result;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Mihomo {
    pub player: Player,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub nickname: String,
    pub level: i32,
    pub avatar: Avatar,
    pub signature: String,
    pub space_info: SpaceInfo,
}

#[derive(Serialize, Deserialize)]
pub struct Avatar {
    pub icon: String,
}

#[derive(Serialize, Deserialize)]
pub struct SpaceInfo {
    pub achievement_count: i32,
}

pub async fn get(uid: i64) -> Result<Mihomo> {
    let url = format!("https://mihomo.shuttleapp.rs/{uid}");

    let mut json: Value = reqwest::get(&url).await?.json().await?;
    if let Some(o) = json.as_object_mut() {
        o.insert(
            "updated_at".to_string(),
            serde_json::to_value(Utc::now().naive_utc())?,
        );
    }

    serde_json::to_writer(&mut File::create(format!("mihomo/{uid}.json"))?, &json)?;

    Ok(serde_json::from_value(json)?)
}
