use std::fs::File;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::Result;

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct Info {
    pub player: Player,
    pub characters: Vec<Character>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Player {
    pub nickname: String,
    pub level: i32,
    pub avatar: Avatar,
    pub signature: String,
    pub space_info: SpaceInfo,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Avatar {
    pub icon: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub avatar_count: i32,
    pub achievement_count: i32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub icon: String,
    pub path: Path,
    pub element: Element,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Path {
    pub icon: String,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Element {
    pub color: String,
    pub icon: String,
}

pub async fn get(uid: i64) -> Result<Info> {
    let url = format!("https://api.mihomo.me/sr_info_parsed/{uid}?lang=en&version=v2");

    let json: Value = reqwest::get(&url).await?.json().await?;
    serde_json::to_writer(&mut File::create(format!("mihomo/{uid}.json"))?, &json)?;

    Ok(serde_json::from_value(json)?)
}
