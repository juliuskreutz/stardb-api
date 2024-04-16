use std::{fs::File, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use anyhow::Result;

use crate::Language;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Mihomo {
    pub player: Player,
    pub updated_at: DateTime<Utc>,
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

pub async fn get_whole(uid: i32, language: Language) -> Result<Value> {
    let path = format!("mihomo/{language}_{uid}.json");

    if PathBuf::from(&path).exists() {
        Ok(serde_json::from_reader(File::open(&path)?)?)
    } else {
        update_and_get_whole(uid, language).await
    }
}

pub async fn update_and_get(uid: i32, language: Language) -> Result<Mihomo> {
    Ok(serde_json::from_value(
        update_and_get_whole(uid, language).await?,
    )?)
}

pub async fn update_and_get_whole(uid: i32, language: Language) -> Result<Value> {
    let url = format!("https://api.mihomo.me/sr_info_parsed/{uid}?lang={language}&version=v2");

    let mut json: Value = reqwest::get(&url).await?.json().await?;
    if let Some(o) = json.as_object_mut() {
        o.insert("updated_at".to_string(), serde_json::to_value(Utc::now())?);
    }

    if serde_json::from_value::<Mihomo>(json.clone()).is_ok() {
        serde_json::to_writer(
            &mut File::create(format!("mihomo/{language}_{uid}.json"))?,
            &json,
        )?;
    }

    Ok(json)
}
