use std::{fs::File, path::PathBuf};

use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use utoipa::ToSchema;

use anyhow::Result;

use crate::{database, Language};

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

pub async fn get(uid: i32, language: Language, pool: &PgPool) -> Result<Value> {
    let path = format!("mihomo/{}_{uid}.json", language.mihomo());

    if PathBuf::from(&path).exists() {
        Ok(serde_json::from_reader(File::open(&path)?)?)
    } else {
        update_and_get(uid, language, pool).await
    }
}

pub async fn update_and_get(uid: i32, language: Language, pool: &PgPool) -> Result<Value> {
    let now = Utc::now();

    let url = format!(
        "https://api.mihomo.me/sr_info_parsed/{uid}?lang={}&version=v2",
        language.mihomo()
    );

    let mut json: Value = reqwest::get(&url).await?.json().await?;
    if let Some(o) = json.as_object_mut() {
        o.insert("updated_at".to_string(), serde_json::to_value(now)?);
    }

    if serde_json::from_value::<Mihomo>(json.clone()).is_ok() {
        serde_json::to_writer(
            &mut File::create(format!("mihomo/{}_{uid}.json", language.mihomo()))?,
            &json,
        )?;
    }

    let mihomo: Mihomo = serde_json::from_value(json.clone())?;

    let re = Regex::new(r"<[^>]*>")?;

    let name = re
        .replace_all(&mihomo.player.nickname, |_: &Captures| "")
        .to_string();
    let region = match uid.to_string().chars().next() {
        Some('6') => "na",
        Some('7') => "eu",
        Some('8') | Some('9') => "asia",
        _ => "cn",
    }
    .to_string();
    let level = mihomo.player.level;
    let avatar_icon = mihomo.player.avatar.icon.clone();
    let signature = re
        .replace_all(&mihomo.player.signature, |_: &Captures| "")
        .to_string();
    let achievement_count = mihomo.player.space_info.achievement_count;
    let updated_at = mihomo.updated_at;
    let timestamp = database::achievement_scores::get_timestamp_by_uid(uid, pool)
        .await
        .ok()
        .and_then(|sd| {
            if sd.achievement_count == achievement_count {
                Some(sd.timestamp)
            } else {
                None
            }
        })
        .unwrap_or(
            now + match region.as_str() {
                "na" => chrono::Duration::try_hours(-5).unwrap(),
                "eu" => chrono::Duration::try_hours(1).unwrap(),
                _ => chrono::Duration::try_hours(8).unwrap(),
            },
        );

    let db_mihomo = database::mihomo::DbMihomo {
        uid,
        region,
        name,
        level,
        signature,
        avatar_icon,
        achievement_count,
        updated_at,
    };

    database::mihomo::set(&db_mihomo, pool).await?;

    let db_score_achievement = database::achievement_scores::DbScoreAchievement {
        uid,
        timestamp,
        ..Default::default()
    };

    database::achievement_scores::set(&db_score_achievement, pool).await?;

    Ok(json)
}
