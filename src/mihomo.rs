use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Serialize, Deserialize)]
pub struct MihomoData {
    pub player: Player,
    pub characters: Vec<Character>,
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
    pub avatar_count: i32,
    pub achievement_count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub icon: String,
    pub path: Path,
    pub element: Element,
}

#[derive(Serialize, Deserialize)]
pub struct Path {
    pub icon: String,
}

#[derive(Serialize, Deserialize)]
pub struct Element {
    pub color: String,
    pub icon: String,
}

pub async fn get(uid: i64) -> Result<MihomoData> {
    let url = format!("https://api.mihomo.me/sr_info_parsed/{uid}?lang=en&version=v2");

    Ok(reqwest::get(&url).await?.json::<MihomoData>().await?)
}
