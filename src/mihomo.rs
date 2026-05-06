use std::{fs::File, path::PathBuf};

use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use utoipa::ToSchema;

use anyhow::{anyhow, Context, Result};

use crate::{database, Language};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Mihomo {
    pub player: Player,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Player {
    pub nickname: String,
    pub level: i32,
    pub avatar: Avatar,
    pub signature: String,
    pub space_info: SpaceInfo,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Avatar {
    pub icon: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SpaceInfo {
    pub achievement_count: i32,
}

fn cache_path(language: &Language, uid: i32) -> String {
    format!("mihomo/{}_{uid}.br", language.mihomo())
}

fn load_cached(language: &Language, uid: i32) -> Result<Option<Mihomo>> {
    let path = cache_path(language, uid);
    if PathBuf::from(&path).exists() {
        let decompressor = brotli::Decompressor::new(File::open(&path)?, 4096);
        Ok(Some(serde_json::from_reader(decompressor)?))
    } else {
        Ok(None)
    }
}

async fn fetch_json(url: &str, uid: i32, language: Language, label: &str) -> Result<Option<Value>> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("{label} request failed for uid {uid} language {language}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(
            uid,
            language = %language,
            url = %url,
            status = %status,
            body = %body,
            label,
            "response was not successful"
        );
        return Ok(None);
    }

    let text = response.text().await.with_context(|| {
        format!("{label} response text retrieval failed for uid {uid} language {language}")
    })?;

    match serde_json::from_str(&text) {
        Ok(json) => Ok(Some(json)),
        Err(err) => {
            warn!(
                uid,
                language = %language,
                url = %url,
                status = %status,
                body = %text,
                label,
                error = %err,
                "response decode failed"
            );
            Err(anyhow!("mihomo response body parsing failed")).with_context(|| {
                format!("{label} json decode failed for uid {uid} language {language}")
            })
        }
    }
}

pub async fn get(uid: i32, language: Language, pool: &PgPool) -> Result<Option<Value>> {
    let path = cache_path(&language, uid);

    if PathBuf::from(&path).exists() {
        let decompressor = brotli::Decompressor::new(File::open(&path)?, 4096);
        let cached_json: Value = serde_json::from_reader(decompressor)?;
        let cached: Mihomo = serde_json::from_value(cached_json.clone())?;

        let should_update = if language == Language::En {
            false
        } else {
            match load_cached(&Language::En, uid)? {
                Some(en) => cached.updated_at < en.updated_at,
                None => true,
            }
        };

        if should_update {
            update_and_get(uid, language, pool).await
        } else {
            Ok(Some(cached_json))
        }
    } else {
        update_and_get(uid, language, pool).await
    }
}

pub async fn update_and_get(uid: i32, language: Language, pool: &PgPool) -> Result<Option<Value>> {
    let now = Utc::now();
    debug!(uid, language = %language, "mihomo update_and_get start");

    let url = format!(
        "https://api.mihomo.me/sr_info_parsed/{uid}?lang={}&version=v2",
        language.mihomo()
    );

    let mut json = match fetch_json(&url, uid, language, "localized").await? {
        Some(json) => json,
        None => return Ok(None),
    };

    debug!(uid, language = %language, "fetched mihomo payload");
    if let Some(o) = json.as_object_mut() {
        o.insert("updated_at".to_string(), serde_json::to_value(now)?);
    }

    let (en_json, is_english) = if language == Language::En {
        debug!(uid, language = %language, "using fetched payload as english payload");
        (json.clone(), true)
    } else {
        let en_url = format!("https://api.mihomo.me/sr_info_parsed/{uid}?lang=en&version=v2",);
        debug!(uid, language = %language, url = %en_url, "fetching english mihomo payload");

        let mut en_json = match fetch_json(&en_url, uid, language, "english").await? {
            Some(json) => json,
            None => return Ok(None),
        };

        debug!(uid, language = %language, "fetched english mihomo payload");
        if let Some(o) = en_json.as_object_mut() {
            o.insert("updated_at".to_string(), serde_json::to_value(now)?);
        }
        (en_json, false)
    };

    if serde_json::from_value::<Mihomo>(json.clone()).is_ok() {
        let file = File::create(cache_path(&language, uid))?;

        let writer = brotli::CompressorWriter::new(file, 4096, 4, 22);

        serde_json::to_writer(writer, &json)?;
        debug!(uid, language = %language, "cached mihomo payload");
    } else {
        debug!(uid, language = %language, "skipped caching localized mihomo payload");
    }

    if !is_english && serde_json::from_value::<Mihomo>(en_json.clone()).is_ok() {
        let file = File::create(cache_path(&Language::En, uid))?;

        let writer = brotli::CompressorWriter::new(file, 4096, 4, 22);

        serde_json::to_writer(writer, &en_json)?;
        debug!(uid, language = %language, "cached english mihomo payload");
    } else if !is_english {
        debug!(uid, language = %language, "skipped caching english mihomo payload");
    }

    let mihomo: Mihomo = serde_json::from_value(en_json)?;

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
    debug!(uid, language = %language, timestamp = %timestamp, "mihomo update_and_get complete");

    Ok(Some(json))
}
